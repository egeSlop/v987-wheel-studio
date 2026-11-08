//! Where dashboard data comes from. Every source turns its game's format into one flat JSON
//! object with the field names in docs/DASH.md and hands it to the hub.
//!
//! - FFB server /stream: FS22_WheelFFB and any other mod that already talks to the FFB server.
//! - UDP 47822 (JSON datagrams): the ETS2/ATS plugin, and any community tool.
//! - WebSocket /feed on the dashboard port: community mods that prefer WebSocket.
//! - DiRT Rally 2.0: the game's own UDP output (extradata=3) on 20777.
//! - Assetto Corsa: the game's UDP remote telemetry on 9996 (we handshake with it).

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use super::Hub;

pub const FEED_UDP_PORT: u16 = 47822;
pub const DR2_PORT: u16 = 20777;
pub const AC_PORT: u16 = 9996;

pub fn spawn_all(hub: Arc<Hub>, stop: Arc<AtomicBool>) {
    let h = hub.clone();
    let s = stop.clone();
    std::thread::Builder::new().name("dash-ffb".into()).spawn(move || ffb_stream(h, s)).ok();
    let h = hub.clone();
    let s = stop.clone();
    std::thread::Builder::new().name("dash-udp".into()).spawn(move || udp_feed(h, s)).ok();
    let h = hub.clone();
    let s = stop.clone();
    std::thread::Builder::new().name("dash-dr2".into()).spawn(move || dr2(h, s)).ok();
    std::thread::Builder::new().name("dash-ac".into()).spawn(move || ac(hub, stop)).ok();
}

/// Applies a JSON message from a community feed (UDP or WebSocket /feed).
pub fn accept_feed(hub: &Hub, source: &str, text: &str) -> Result<(), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let Value::Object(mut m) = v else { return Err("expected a JSON object".into()) };
    match m.get("type").and_then(Value::as_str).unwrap_or("dash") {
        "dash" | "telemetry" => {}
        other => return Err(format!("unknown type \"{other}\"")),
    }
    m.remove("type");
    let game = m.get("game").and_then(Value::as_str).unwrap_or("").to_string();
    if game.is_empty() {
        return Err("\"game\" is required".into());
    }
    normalize(&mut m);
    hub.update(source, &game, m);
    Ok(())
}

/// Fills derived fields every template uses (speedKmh from speed, cruiseOn from cruise...).
pub fn normalize(m: &mut Map<String, Value>) {
    let f = |m: &Map<String, Value>, k: &str| m.get(k).and_then(Value::as_f64);
    if !m.contains_key("speedKmh") {
        if let Some(s) = f(m, "speed") {
            m.insert("speedKmh".into(), json!((s * 3.6).abs()));
        }
    }
    if !m.contains_key("cruiseOn") {
        if let Some(c) = f(m, "cruise") {
            m.insert("cruiseOn".into(), json!(c > 0.5));
        }
    }
    if let Some(g) = m.get("gear").cloned() {
        // Numbers are accepted too: 0 = N, negative = reverse.
        if let Some(n) = g.as_i64() {
            let s = match n {
                0 => "N".to_string(),
                n if n < 0 => if n == -1 { "R".into() } else { format!("R{}", -n) },
                n => n.to_string(),
            };
            m.insert("gear".into(), json!(s));
        }
    }
    if !m.contains_key("active") {
        m.insert("active".into(), json!(true));
    }
}

// ---------------------------------------------------------------- FFB server /stream

fn ffb_stream(hub: Arc<Hub>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        let listen = ffb_server::config::load().listen;
        let url = format!("ws://{listen}/stream");
        match connect_ws(&listen, &url) {
            Ok(mut ws) => {
                hub.set_source_state("ffb", true, None);
                let mut game_name = String::new();
                loop {
                    if stop.load(Ordering::Relaxed) {
                        let _ = ws.close(None);
                        return;
                    }
                    match ws.read() {
                        Ok(tungstenite::Message::Text(t)) => {
                            let Ok(v) = serde_json::from_str::<Value>(t.as_str()) else { continue };
                            match v.get("type").and_then(Value::as_str) {
                                Some("frame") => {
                                    let game = v.get("game").and_then(Value::as_str).unwrap_or("").to_string();
                                    if !game.is_empty() {
                                        game_name = game;
                                    }
                                    if let Some(Value::Object(t)) = v.get("telemetry") {
                                        let mut m = t.clone();
                                        for k in ["wheels", "seq", "t", "v", "steerInput", "lockOver", "vLat", "vUp", "vFwd", "yawRate", "pitchRate", "rollRate", "load", "mass"] {
                                            m.remove(k);
                                        }
                                        normalize(&mut m);
                                        let g = m.get("game").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| game_name.clone());
                                        hub.update("ffb", &g, m);
                                    }
                                }
                                Some("status") => {
                                    // Not driving: frames stop. Tell the hub the game is idle.
                                    let mode = v.get("mode").and_then(Value::as_str).unwrap_or("");
                                    let g = v.get("game").and_then(Value::as_str).unwrap_or("").to_string();
                                    if mode == "idle" && !g.is_empty() {
                                        let mut m = Map::new();
                                        m.insert("active".into(), json!(false));
                                        hub.update_idle("ffb", &g, m);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(_) => {}
                        Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
                        Err(_) => break,
                    }
                }
                hub.set_source_state("ffb", false, None);
            }
            Err(e) => hub.set_source_state("ffb", false, Some(e)),
        }
        sleep_unless(&stop, Duration::from_secs(2));
    }
}

fn connect_ws(listen: &str, url: &str) -> Result<tungstenite::WebSocket<std::net::TcpStream>, String> {
    use std::net::ToSocketAddrs;
    let addr = listen.to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("bad address")?;
    let tcp = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).map_err(|e| e.to_string())?;
    tcp.set_read_timeout(Some(Duration::from_millis(300))).ok();
    let (ws, _) = tungstenite::client(url, tcp).map_err(|e| e.to_string())?;
    Ok(ws)
}

fn sleep_unless(stop: &AtomicBool, d: Duration) {
    let until = Instant::now() + d;
    while Instant::now() < until && !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }
}

// ---------------------------------------------------------------- UDP JSON feed

fn udp_feed(hub: Arc<Hub>, stop: Arc<AtomicBool>) {
    let sock = match UdpSocket::bind(("127.0.0.1", FEED_UDP_PORT)) {
        Ok(s) => s,
        Err(e) => {
            hub.set_source_state("udp", false, Some(format!("udp {FEED_UDP_PORT}: {e}")));
            return;
        }
    };
    sock.set_read_timeout(Some(Duration::from_millis(300))).ok();
    hub.set_source_state("udp", true, None);
    let mut buf = vec![0u8; 65536];
    while !stop.load(Ordering::Relaxed) {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                if let Err(e) = accept_feed(&hub, "udp", text) {
                    hub.set_source_state("udp", true, Some(e));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- DiRT Rally 2.0

fn dr2(hub: Arc<Hub>, stop: Arc<AtomicBool>) {
    let sock = match UdpSocket::bind(("127.0.0.1", DR2_PORT)) {
        Ok(s) => s,
        Err(e) => {
            hub.set_source_state("dr2", false, Some(format!("udp {DR2_PORT}: {e}")));
            return;
        }
    };
    sock.set_read_timeout(Some(Duration::from_millis(300))).ok();
    hub.set_source_state("dr2", true, None);
    let mut buf = [0u8; 2048];
    while !stop.load(Ordering::Relaxed) {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            if let Some(m) = parse_dr2(&buf[..n]) {
                hub.update("dr2", "dr2", m);
            }
        }
    }
}

/// Codemasters "extradata=3" packet: 66 little-endian f32.
pub fn parse_dr2(b: &[u8]) -> Option<Map<String, Value>> {
    if b.len() < 64 * 4 {
        return None;
    }
    let f = |i: usize| f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap()) as f64;
    let gear = f(33);
    let gear = if gear < -0.5 || gear > 9.5 { "R".to_string() } else if gear < 0.5 { "N".to_string() } else { format!("{}", gear.round() as i64) };
    let mut m = Map::new();
    m.insert("name".into(), json!("DiRT Rally 2.0"));
    m.insert("speedKmh".into(), json!(f(7).abs() * 3.6));
    m.insert("gear".into(), json!(gear));
    m.insert("rpm".into(), json!(f(37) * 10.0));
    if b.len() >= 66 * 4 {
        m.insert("maxRpm".into(), json!(f(63) * 10.0));
        m.insert("gearsForward".into(), json!(f(65)));
    }
    m.insert("throttle".into(), json!(f(29)));
    m.insert("brake".into(), json!(f(31)));
    m.insert("clutch".into(), json!(f(32)));
    m.insert("lapTime".into(), json!(f(1)));
    m.insert("distance".into(), json!(f(2).max(0.0) / 1000.0));
    let track = f(61);
    if track > 1.0 {
        m.insert("progress".into(), json!((f(2) / track).clamp(0.0, 1.0)));
    }
    m.insert("position".into(), json!(f(39)));
    m.insert("engine".into(), json!(f(37) > 1.0));
    m.insert("active".into(), json!(true));
    Some(m)
}

// ---------------------------------------------------------------- Assetto Corsa

fn ac(hub: Arc<Hub>, stop: Arc<AtomicBool>) {
    let sock = match UdpSocket::bind(("127.0.0.1", 0)) {
        Ok(s) => s,
        Err(e) => {
            hub.set_source_state("ac", false, Some(e.to_string()));
            return;
        }
    };
    sock.set_read_timeout(Some(Duration::from_millis(300))).ok();
    let target = ("127.0.0.1", AC_PORT);
    let handshake = |op: i32| {
        let mut p = [0u8; 12];
        p[0..4].copy_from_slice(&1i32.to_le_bytes()); // identifier: any
        p[4..8].copy_from_slice(&1i32.to_le_bytes()); // version
        p[8..12].copy_from_slice(&op.to_le_bytes()); // 0 handshake, 1 subscribe update, 3 dismiss
        p
    };
    hub.set_source_state("ac", true, None);
    let mut buf = [0u8; 2048];
    let mut last_data = Instant::now() - Duration::from_secs(60);
    let mut last_hello = Instant::now() - Duration::from_secs(60);
    let mut car = String::new();
    while !stop.load(Ordering::Relaxed) {
        if last_data.elapsed() > Duration::from_secs(2) && last_hello.elapsed() > Duration::from_secs(2) {
            last_hello = Instant::now();
            let _ = sock.send_to(&handshake(0), target);
        }
        match sock.recv_from(&mut buf) {
            Ok((n, _)) if n >= 400 && n < 600 => {
                // Handshake reply: carName[50] wchar, driverName[50], id, version, trackName[50], trackConfig[50].
                car = utf16z(&buf[0..100]);
                let _ = sock.send_to(&handshake(1), target);
                last_data = Instant::now();
            }
            Ok((n, _)) if n >= 300 => {
                last_data = Instant::now();
                if let Some(mut m) = parse_ac(&buf[..n]) {
                    if !car.is_empty() {
                        m.insert("vehicle".into(), json!(car));
                    }
                    hub.update("ac", "ac", m);
                }
            }
            _ => {}
        }
    }
    let _ = sock.send_to(&handshake(3), target);
}

fn utf16z(b: &[u8]) -> String {
    let u: Vec<u16> = b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect();
    let s = String::from_utf16_lossy(&u);
    // AC pads names with '%'.
    s.split('%').next().unwrap_or("").trim().to_string()
}

/// RTCarInfo (328 bytes).
pub fn parse_ac(b: &[u8]) -> Option<Map<String, Value>> {
    if b.len() < 80 {
        return None;
    }
    let f = |o: usize| f32::from_le_bytes(b[o..o + 4].try_into().unwrap()) as f64;
    let i = |o: usize| i32::from_le_bytes(b[o..o + 4].try_into().unwrap());
    let gear = i(76); // 0 = R, 1 = N, 2 = 1st
    let gear = match gear {
        0 => "R".to_string(),
        1 => "N".to_string(),
        g => (g - 1).to_string(),
    };
    let mut m = Map::new();
    m.insert("name".into(), json!("Assetto Corsa"));
    m.insert("speedKmh".into(), json!(f(8).abs()));
    m.insert("absOn".into(), json!(b[20] != 0));
    m.insert("tcOn".into(), json!(b[23] != 0));
    m.insert("inPit".into(), json!(b[24] != 0));
    m.insert("limiter".into(), json!(b[25] != 0));
    m.insert("lapTime".into(), json!(i(40) as f64 / 1000.0));
    let last = i(44);
    let best = i(48);
    if last > 0 {
        m.insert("lastLap".into(), json!(last as f64 / 1000.0));
    }
    if best > 0 {
        m.insert("bestLap".into(), json!(best as f64 / 1000.0));
    }
    m.insert("lap".into(), json!(i(52) + 1));
    m.insert("throttle".into(), json!(f(56)));
    m.insert("brake".into(), json!(f(60)));
    m.insert("clutch".into(), json!(f(64)));
    m.insert("rpm".into(), json!(f(68)));
    m.insert("gear".into(), json!(gear));
    m.insert("engine".into(), json!(f(68) > 1.0));
    m.insert("active".into(), json!(true));
    Some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dr2_packet() {
        let mut v = vec![0f32; 66];
        v[7] = 25.0; // m/s
        v[33] = 3.0;
        v[37] = 550.0; // 5500 rpm
        v[63] = 800.0;
        v[2] = 500.0;
        v[61] = 10000.0;
        let b: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        let m = parse_dr2(&b).unwrap();
        assert_eq!(m["gear"], "3");
        assert!((m["speedKmh"].as_f64().unwrap() - 90.0).abs() < 0.01);
        assert_eq!(m["rpm"].as_f64().unwrap(), 5500.0);
        assert!((m["progress"].as_f64().unwrap() - 0.05).abs() < 1e-6);
        v[33] = 10.0;
        let b: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        assert_eq!(parse_dr2(&b).unwrap()["gear"], "R");
    }

    #[test]
    fn ac_packet() {
        let mut b = vec![0u8; 328];
        b[8..12].copy_from_slice(&123.5f32.to_le_bytes());
        b[20] = 1;
        b[44..48].copy_from_slice(&83456i32.to_le_bytes());
        b[52..56].copy_from_slice(&2i32.to_le_bytes());
        b[68..72].copy_from_slice(&6400f32.to_le_bytes());
        b[76..80].copy_from_slice(&4i32.to_le_bytes());
        let m = parse_ac(&b).unwrap();
        assert_eq!(m["gear"], "3");
        assert_eq!(m["lap"], 3);
        assert_eq!(m["absOn"], true);
        assert!((m["lastLap"].as_f64().unwrap() - 83.456).abs() < 1e-9);
    }

    #[test]
    fn normalize_fills_derived_fields() {
        let mut m: Map<String, Value> = serde_json::from_str(r#"{"speed":-5,"cruise":40,"gear":-2}"#).unwrap();
        normalize(&mut m);
        assert_eq!(m["speedKmh"], 18.0);
        assert_eq!(m["cruiseOn"], true);
        assert_eq!(m["gear"], "R2");
        assert_eq!(m["active"], true);
    }
}
