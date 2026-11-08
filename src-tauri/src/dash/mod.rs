//! Tablet dashboard: a small web server on the LAN. A tablet opens it in the browser and gets
//! a car-radio style screen with live data from the game and a panel of buttons that press
//! the game's keys on this PC.
//!
//! Port 47821 (TCP): the page, WebSocket /ws for tablets and /feed for community mods.
//! Port 47822 (UDP, localhost): JSON datagrams from game plugins. See docs/DASH.md.

pub mod keys;
pub mod profiles;
pub mod sources;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

mod web {
    include!(concat!(env!("OUT_DIR"), "/dash_web.rs"));
}
mod ets2_bin {
    include!(concat!(env!("OUT_DIR"), "/ets2_plugin.rs"));
}

pub const DEFAULT_PORT: u16 = 47821;

// ---------------------------------------------------------------- settings

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Start with Wheel Studio (and with the headless server).
    pub autostart: bool,
    pub port: u16,
    /// Tablets must enter this PIN (or scan the QR code) before buttons work.
    pub pin: String,
    pub require_pin: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { autostart: true, port: DEFAULT_PORT, pin: new_pin(), require_pin: true }
    }
}

pub fn config_path() -> PathBuf {
    ffb_server::config::path().parent().map(|p| p.to_path_buf()).unwrap_or_default().join("dash.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path).ok().and_then(|t| toml::from_str::<Config>(&t).ok()) {
        Some(mut c) => {
            if c.pin.len() != 6 || !c.pin.chars().all(|c| c.is_ascii_digit()) {
                c.pin = new_pin();
                let _ = save_config(&c);
            }
            c
        }
        None => {
            let c = Config::default();
            let _ = save_config(&c);
            c
        }
    }
}

pub fn save_config(c: &Config) -> std::io::Result<()> {
    let path = config_path();
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let body = toml::to_string_pretty(c).unwrap_or_default();
    std::fs::write(path, format!("# V987 Wheel Studio tablet dashboard. Edited by the app.\n{body}"))
}

pub fn new_pin() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos());
    format!("{:06}", h.finish() % 1_000_000)
}

// ---------------------------------------------------------------- hub

struct Feed {
    game: String,
    at: Instant,
    active: bool,
    data: Map<String, Value>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub up: bool,
    pub error: Option<String>,
    /// Milliseconds since this source last delivered data.
    pub last_data_ms: Option<u64>,
    pub game: Option<String>,
}

/// Latest data from every source; picks the game being played.
pub struct Hub {
    feeds: Mutex<HashMap<String, Feed>>,
    sources: Mutex<HashMap<String, SourceInfo>>,
    version: AtomicU64,
}

impl Hub {
    fn new() -> Self {
        Self { feeds: Mutex::new(HashMap::new()), sources: Mutex::new(HashMap::new()), version: AtomicU64::new(0) }
    }

    pub fn update(&self, source: &str, game: &str, data: Map<String, Value>) {
        let active = data.get("active").and_then(Value::as_bool).unwrap_or(true);
        self.feeds.lock().unwrap().insert(source.into(), Feed { game: game.into(), at: Instant::now(), active, data });
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    /// The game is still there but not driving (menu, pause): keep the last values, mark idle.
    pub fn update_idle(&self, source: &str, game: &str, data: Map<String, Value>) {
        let mut feeds = self.feeds.lock().unwrap();
        match feeds.get_mut(source) {
            Some(f) if f.game == game || f.game.is_empty() => {
                if f.active || f.at.elapsed() > Duration::from_secs(1) {
                    f.active = false;
                    f.data.insert("active".into(), json!(false));
                    f.at = Instant::now();
                }
            }
            _ => {
                feeds.insert(source.into(), Feed { game: game.into(), at: Instant::now(), active: false, data });
            }
        }
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_source_state(&self, source: &str, up: bool, error: Option<String>) {
        let mut s = self.sources.lock().unwrap();
        let e = s.entry(source.into()).or_default();
        e.up = up;
        e.error = error;
    }

    /// (source, game, active, age, data) of the game the player is most likely in.
    fn current(&self) -> Option<(String, String, bool, Duration, Map<String, Value>)> {
        let feeds = self.feeds.lock().unwrap();
        let fresh = |f: &&Feed, max: u64| f.at.elapsed() < Duration::from_secs(max);
        let pick = feeds
            .iter()
            .filter(|(_, f)| f.active && fresh(f, 3))
            .max_by_key(|(_, f)| f.at)
            .or_else(|| feeds.iter().filter(|(_, f)| fresh(f, 60)).max_by_key(|(_, f)| f.at))?;
        let (src, f) = pick;
        Some((src.clone(), f.game.clone(), f.active && fresh(&f, 3), f.at.elapsed(), f.data.clone()))
    }

    fn sources(&self) -> HashMap<String, SourceInfo> {
        let feeds = self.feeds.lock().unwrap();
        let mut s = self.sources.lock().unwrap().clone();
        for (k, f) in feeds.iter() {
            let e = s.entry(k.clone()).or_default();
            e.last_data_ms = Some(f.at.elapsed().as_millis() as u64);
            e.game = Some(f.game.clone());
        }
        s
    }
}

// ---------------------------------------------------------------- server

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabletInfo {
    pub id: u64,
    pub peer: String,
    pub device: String,
    pub paired: bool,
    pub connected_ms: u64,
    pub presses: u64,
}

struct Tablet {
    info: TabletInfo,
    since: Instant,
    tx: Sender<String>,
}

enum KeyOp {
    Tap(Vec<u16>),
    Down(Vec<u16>),
    Up(Vec<u16>),
}

struct Shared {
    config: Mutex<Config>,
    hub: Arc<Hub>,
    tablets: Mutex<Vec<Tablet>>,
    next_id: AtomicU64,
    profiles: Mutex<(Vec<profiles::Profile>, Vec<String>, SystemTime)>,
    bindings: Mutex<profiles::Bindings>,
    keyboard: keys::Keyboard,
    key_tx: Mutex<Sender<KeyOp>>,
    stop: Arc<AtomicBool>,
    feeds_ws: AtomicU64,
}

pub struct DashServer {
    sh: Arc<Shared>,
    addr: SocketAddr,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub port: u16,
    pub urls: Vec<String>,
    pub pair_url: String,
    pub qr_svg: String,
    pub tablets: Vec<TabletInfo>,
    pub sources: HashMap<String, SourceInfo>,
    pub game: Option<Value>,
    pub keyboard_ready: bool,
    pub keyboard_error: Option<String>,
    pub profiles: Vec<ProfileInfo>,
    pub profile_errors: Vec<String>,
    pub profiles_dir: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub origin: String,
    pub buttons: Vec<ButtonInfo>,
    pub setup: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct ButtonInfo {
    pub id: String,
    pub label: String,
    pub labels: HashMap<String, String>,
    pub key: String,
    pub action: String,
}

impl DashServer {
    pub fn start(config: Config) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", config.port))
            .map_err(|e| anyhow::anyhow!("port {} is busy or blocked ({e}). Is another Wheel Studio running?", config.port))?;
        let addr = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let hub = Arc::new(Hub::new());
        let (key_tx, key_rx) = channel();
        let listing = profiles::load();
        let sh = Arc::new(Shared {
            config: Mutex::new(config),
            hub: hub.clone(),
            tablets: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            profiles: Mutex::new((listing.profiles, listing.errors, profiles_mtime())),
            bindings: Mutex::new(profiles::Bindings::default()),
            keyboard: keys::Keyboard::new(),
            key_tx: Mutex::new(key_tx),
            stop: stop.clone(),
            feeds_ws: AtomicU64::new(0),
        });
        sources::spawn_all(hub, stop.clone());
        std::thread::Builder::new().name("dash-names".into()).spawn(profiles::learn_station_names).ok();

        let s = sh.clone();
        std::thread::Builder::new().name("dash-keys".into()).spawn(move || key_worker(s, key_rx))?;
        let s = sh.clone();
        std::thread::Builder::new().name("dash-push".into()).spawn(move || broadcaster(s))?;
        let s = sh.clone();
        listener.set_nonblocking(true)?;
        std::thread::Builder::new().name("dash-http".into()).spawn(move || {
            while !s.stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, peer)) => {
                        let s2 = s.clone();
                        std::thread::spawn(move || {
                            let _ = stream.set_nonblocking(false);
                            handle_conn(s2, stream, peer);
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(40)),
                    Err(_) => std::thread::sleep(Duration::from_millis(200)),
                }
            }
        })?;
        eprintln!("[dash] tablet dashboard on http://{}:{}", lan_ip().unwrap_or_else(|| "0.0.0.0".into()), addr.port());
        Ok(Self { sh, addr })
    }

    pub fn stop(self) {
        self.sh.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], self.addr.port())), Duration::from_millis(100));
    }

    pub fn config(&self) -> Config {
        self.sh.config.lock().unwrap().clone()
    }

    /// Applies new settings. Returns true if the port changed (restart needed).
    pub fn set_config(&self, c: Config) -> std::io::Result<bool> {
        save_config(&c)?;
        let mut cur = self.sh.config.lock().unwrap();
        let restart = cur.port != c.port;
        let pin_changed = cur.pin != c.pin || (c.require_pin && !cur.require_pin);
        *cur = c;
        drop(cur);
        if pin_changed {
            // Everyone pairs again.
            for t in self.sh.tablets.lock().unwrap().iter_mut() {
                t.info.paired = false;
                let _ = t.tx.send(json!({"type":"auth","ok":false,"reason":"pin-changed"}).to_string());
            }
        }
        Ok(restart)
    }

    pub fn reload_profiles(&self) {
        reload_profiles(&self.sh, true);
    }

    pub fn status(&self) -> Status {
        let cfg = self.config();
        let host = lan_ip().unwrap_or_else(|| "127.0.0.1".into());
        let base = format!("http://{host}:{}", self.addr.port());
        let pair_url = if cfg.require_pin { format!("{base}/#pin={}", cfg.pin) } else { format!("{base}/") };
        let (profs, errs, _) = &*self.sh.profiles.lock().unwrap();
        let (cur, _) = current_json(&self.sh);
        let mut sources = self.sh.hub.sources();
        sources.entry("feed".into()).or_default().up = true;
        if let Some(f) = sources.get_mut("feed") {
            f.error = None;
            if self.sh.feeds_ws.load(Ordering::Relaxed) == 0 && f.last_data_ms.is_none() {
                f.up = true;
            }
        }
        Status {
            port: self.addr.port(),
            urls: vec![format!("{base}/"), format!("http://{}:{}/", hostname(), self.addr.port())],
            qr_svg: qr_svg(&pair_url),
            pair_url,
            tablets: self
                .sh
                .tablets
                .lock()
                .unwrap()
                .iter()
                .map(|t| TabletInfo { connected_ms: t.since.elapsed().as_millis() as u64, ..t.info.clone() })
                .collect(),
            sources,
            game: cur,
            keyboard_ready: self.sh.keyboard.ready(),
            keyboard_error: self.sh.keyboard.error(),
            profiles: profs
                .iter()
                .map(|p| ProfileInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    origin: p.origin.clone(),
                    buttons: p
                        .pages
                        .iter()
                        .flat_map(|x| &x.buttons)
                        .map(|b| ButtonInfo { id: b.id.clone(), label: b.label.clone(), labels: b.labels.clone(), key: b.key.clone(), action: b.action.clone() })
                        .collect(),
                    setup: p.setup.clone(),
                })
                .collect(),
            profile_errors: errs.clone(),
            profiles_dir: profiles::user_dir().display().to_string(),
        }
    }

    /// Presses a profile button from the desktop app (the "Test" buttons).
    pub fn test_press(&self, profile: &str, button: &str) -> Result<String, String> {
        let keys = resolve(&self.sh, profile, PressTarget::Button(button))?;
        self.sh.keyboard.tap(&keys.0)?;
        Ok(keys.1)
    }

    pub fn warm_keyboard(&self) -> Result<(), String> {
        self.sh.keyboard.warm_up()
    }
}

fn profiles_mtime() -> SystemTime {
    let mut newest = SystemTime::UNIX_EPOCH;
    if let Ok(rd) = std::fs::read_dir(profiles::user_dir()) {
        for e in rd.flatten() {
            if let Ok(t) = e.metadata().and_then(|m| m.modified()) {
                newest = newest.max(t);
            }
        }
    }
    if let Some(d) = profiles::fs22_profile_dir() {
        if let Ok(t) = std::fs::metadata(d.join("music/streamingInternetRadios.xml")).and_then(|m| m.modified()) {
            newest = newest.max(t);
        }
    }
    newest
}

fn reload_profiles(sh: &Shared, force: bool) {
    let m = profiles_mtime();
    {
        let g = sh.profiles.lock().unwrap();
        if !force && g.2 == m {
            return;
        }
    }
    let l = profiles::load();
    *sh.profiles.lock().unwrap() = (l.profiles, l.errors, m);
    let hello = hello_json(sh, None);
    for t in sh.tablets.lock().unwrap().iter() {
        let mut h = hello.clone();
        h["paired"] = json!(t.info.paired);
        let _ = t.tx.send(h.to_string());
    }
}

fn hello_json(sh: &Shared, paired: Option<bool>) -> Value {
    let (profs, _, _) = &*sh.profiles.lock().unwrap();
    let (stations, fs22) = profiles::stations();
    let cfg = sh.config.lock().unwrap().clone();
    json!({
        "type": "hello",
        "app": crate::APP_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "paired": paired.unwrap_or(!cfg.require_pin),
        "pinRequired": cfg.require_pin,
        "profiles": profs,
        "stations": stations,
        "gameStations": { "fs22": fs22 },
        "keyboardError": sh.keyboard.error(),
    })
}

/// The current game as sent to tablets, plus a change key.
fn current_json(sh: &Shared) -> (Option<Value>, String) {
    let Some((source, game, active, age, data)) = sh.hub.current() else { return (None, String::new()) };
    let (profs, _, _) = &*sh.profiles.lock().unwrap();
    let name = data.get("name").and_then(Value::as_str).map(str::to_string);
    let p = profiles::pick(profs, &game).or_else(|| name.as_deref().and_then(|n| profiles::pick(profs, n)));
    let profile = p.map(|p| p.id.clone()).unwrap_or_else(|| "generic".into());
    let display = name.or_else(|| p.map(|p| p.name.clone())).unwrap_or_else(|| game.clone());
    let v = json!({
        "profile": profile,
        "game": game,
        "name": display,
        "source": source,
        "active": active,
        "ageMs": age.as_millis() as u64,
        "data": data,
    });
    let key = format!("{profile}|{active}|{}", serde_json::to_string(&v["data"]).unwrap_or_default());
    (Some(v), key)
}

fn broadcaster(sh: Arc<Shared>) {
    let mut last_key = String::new();
    let mut last_sent = Instant::now();
    let mut last_reload = Instant::now();
    while !sh.stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(66));
        if last_reload.elapsed() > Duration::from_secs(3) {
            last_reload = Instant::now();
            reload_profiles(&sh, false);
        }
        let (cur, key) = current_json(&sh);
        if key == last_key && last_sent.elapsed() < Duration::from_secs(1) {
            continue;
        }
        last_key = key;
        last_sent = Instant::now();
        let msg = json!({ "type": "state", "game": cur, "t": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }).to_string();
        let mut tabs = sh.tablets.lock().unwrap();
        tabs.retain(|t| t.tx.send(msg.clone()).is_ok());
    }
}

fn key_worker(sh: Arc<Shared>, rx: Receiver<KeyOp>) {
    while let Ok(op) = rx.recv() {
        let r = match op {
            KeyOp::Tap(k) => sh.keyboard.tap(&k),
            KeyOp::Down(k) => sh.keyboard.down(&k),
            KeyOp::Up(k) => sh.keyboard.up(&k),
        };
        if let Err(e) = r {
            eprintln!("[dash] key press failed: {e}");
        }
    }
}

enum PressTarget<'a> {
    Button(&'a str),
    Radio(&'a str),
}

/// Keys for a button or radio control, and a readable description ("lctrl+f (from FS22)").
fn resolve(sh: &Shared, profile: &str, target: PressTarget) -> Result<(Vec<u16>, String), String> {
    let (profs, _, _) = &*sh.profiles.lock().unwrap();
    let p = profs.iter().find(|p| p.id == profile).ok_or_else(|| format!("no profile \"{profile}\""))?;
    let (key, action) = match target {
        PressTarget::Button(id) => {
            let b = p.pages.iter().flat_map(|pg| &pg.buttons).find(|b| b.id == id).ok_or_else(|| format!("no button \"{id}\""))?;
            (b.key.clone(), b.action.clone())
        }
        PressTarget::Radio(what) => {
            let r = &p.radio;
            let k = match what {
                "power" => &r.power,
                "next" => &r.next,
                "prev" => &r.prev,
                "volUp" => &r.vol_up,
                "volDown" => &r.vol_down,
                _ => &None,
            };
            let k = k.as_ref().ok_or_else(|| format!("this profile has no radio {what} key"))?;
            (k.key.clone(), k.action.clone())
        }
    };
    if !p.bindings.is_empty() && !action.is_empty() {
        if let Some(b) = sh.bindings.lock().unwrap().lookup(&p.bindings, &action) {
            if let Ok(k) = keys::parse_combo(&b) {
                return Ok((k, format!("{b} (from your {} bindings)", p.bindings.to_uppercase())));
            }
        }
    }
    if key.is_empty() {
        return Err(format!("\"{action}\" has no keyboard key in the game and no fallback key"));
    }
    keys::parse_combo(&key).map(|k| (k, key))
}

// ---------------------------------------------------------------- connections

fn handle_conn(sh: Arc<Shared>, mut stream: TcpStream, peer: SocketAddr) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    // Peek at the request so a WebSocket upgrade can be handed to tungstenite untouched.
    let mut buf = [0u8; 4096];
    let mut n = 0;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        match stream.peek(&mut buf) {
            Ok(0) => return,
            Ok(k) => {
                n = k;
                if buf[..k].windows(4).any(|w| w == b"\r\n\r\n") || k == buf.len() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return,
        }
    }
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    let mut lines = head.lines();
    let first = lines.next().unwrap_or("");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    let path = target.split(['?', '#']).next().unwrap_or("/");
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string())))
        .collect();
    let header = |k: &str| headers.iter().find(|(h, _)| h == k).map(|(_, v)| v.as_str()).unwrap_or("");
    let upgrade = header("upgrade").eq_ignore_ascii_case("websocket");

    if upgrade && path == "/ws" {
        let device = describe_device(header("user-agent"));
        stream.set_read_timeout(None).ok();
        if let Ok(ws) = tungstenite::accept(stream) {
            tablet_session(sh, ws, peer, device);
        }
        return;
    }
    if upgrade && path == "/feed" {
        stream.set_read_timeout(None).ok();
        if let Ok(ws) = tungstenite::accept(stream) {
            feed_session(sh, ws, peer);
        }
        return;
    }
    // Plain HTTP: consume the request, answer, close.
    let _ = stream.read(&mut buf[..n]);
    if method != "GET" && method != "HEAD" {
        respond(&mut stream, "405 Method Not Allowed", "text/plain", b"GET only", method == "HEAD");
        return;
    }
    let file = match path {
        "/" | "/index.html" => "index.html",
        p => p.trim_start_matches('/'),
    };
    match web::DASH_WEB.iter().find(|(f, _)| *f == file) {
        Some((f, body)) => respond(&mut stream, "200 OK", mime(f), body, method == "HEAD"),
        None => respond(&mut stream, "404 Not Found", "text/plain", b"not found", method == "HEAD"),
    }
}

fn respond(stream: &mut TcpStream, status: &str, ctype: &str, body: &[u8], head_only: bool) {
    let hdr = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(hdr.as_bytes());
    if !head_only {
        let _ = stream.write_all(body);
    }
    let _ = stream.flush();
}

fn mime(f: &str) -> &'static str {
    match f.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webmanifest" => "application/manifest+json",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn describe_device(ua: &str) -> String {
    let os = if ua.contains("iPad") {
        "iPad"
    } else if ua.contains("iPhone") {
        "iPhone"
    } else if ua.contains("Android") {
        if ua.contains("Mobile") { "Android phone" } else { "Android tablet" }
    } else if ua.contains("Windows") {
        "Windows"
    } else if ua.contains("Mac OS") {
        "Mac"
    } else if ua.contains("Linux") {
        "Linux"
    } else {
        "Browser"
    };
    let br = if ua.contains("Firefox") {
        "Firefox"
    } else if ua.contains("SamsungBrowser") {
        "Samsung Internet"
    } else if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Chrome") {
        "Chrome"
    } else if ua.contains("Safari") {
        "Safari"
    } else {
        ""
    };
    if br.is_empty() { os.into() } else { format!("{os} · {br}") }
}

type Ws = tungstenite::WebSocket<TcpStream>;

fn is_timeout(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(io) if matches!(io.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut))
}

fn tablet_session(sh: Arc<Shared>, mut ws: Ws, peer: SocketAddr, device: String) {
    ws.get_ref().set_read_timeout(Some(Duration::from_millis(30))).ok();
    let id = sh.next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = channel::<String>();
    let open = !sh.config.lock().unwrap().require_pin;
    sh.tablets.lock().unwrap().push(Tablet {
        info: TabletInfo { id, peer: peer.ip().to_string(), device, paired: open, connected_ms: 0, presses: 0 },
        since: Instant::now(),
        tx: tx.clone(),
    });
    let _ = ws.send(tungstenite::Message::text(hello_json(&sh, Some(open)).to_string()));
    let (cur, _) = current_json(&sh);
    let _ = ws.send(tungstenite::Message::text(json!({"type":"state","game":cur}).to_string()));
    // Keys this tablet is holding down; released if it disconnects mid-press.
    let mut held: HashMap<String, Vec<u16>> = HashMap::new();
    let mut fails = 0u32;
    let mut last_ping = Instant::now();
    'outer: loop {
        if sh.stop.load(Ordering::Relaxed) {
            let _ = ws.close(None);
            break;
        }
        while let Ok(m) = rx.try_recv() {
            if ws.send(tungstenite::Message::text(m)).is_err() {
                break 'outer;
            }
        }
        if last_ping.elapsed() > Duration::from_secs(5) {
            last_ping = Instant::now();
            if ws.send(tungstenite::Message::Ping(Vec::new().into())).is_err() {
                break;
            }
        }
        match ws.read() {
            Ok(tungstenite::Message::Text(t)) => {
                let reply = on_tablet_message(&sh, id, t.as_str(), &mut held, &mut fails);
                if let Some(r) = reply {
                    if ws.send(tungstenite::Message::text(r.to_string())).is_err() {
                        break;
                    }
                }
            }
            Ok(tungstenite::Message::Close(_)) => break,
            Ok(_) => {}
            Err(e) if is_timeout(&e) => {}
            Err(_) => break,
        }
    }
    let tx = sh.key_tx.lock().unwrap().clone();
    for (_, k) in held.drain() {
        let _ = tx.send(KeyOp::Up(k));
    }
    sh.tablets.lock().unwrap().retain(|t| t.info.id != id);
}

fn on_tablet_message(sh: &Shared, id: u64, text: &str, held: &mut HashMap<String, Vec<u16>>, fails: &mut u32) -> Option<Value> {
    let v: Value = serde_json::from_str(text).ok()?;
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("");
    let paired = || sh.tablets.lock().unwrap().iter().find(|t| t.info.id == id).is_some_and(|t| t.info.paired);
    match ty {
        "auth" => {
            let pin = v.get("pin").and_then(Value::as_str).unwrap_or("");
            let cfg = sh.config.lock().unwrap().clone();
            // Slow down guessing: after 5 wrong PINs this connection only gets one try every 3 s.
            if *fails >= 5 {
                std::thread::sleep(Duration::from_secs(3));
            }
            let ok = !cfg.require_pin || pin == cfg.pin;
            if !ok {
                *fails += 1;
            }
            if let Some(t) = sh.tablets.lock().unwrap().iter_mut().find(|t| t.info.id == id) {
                t.info.paired = ok;
            }
            Some(json!({"type":"auth","ok":ok,"reason": if ok { "" } else { "wrong-pin" }}))
        }
        "press" | "radio" => {
            let profile = v.get("profile").and_then(Value::as_str).unwrap_or("");
            let rid = v.get("id").cloned().unwrap_or(Value::Null);
            if !paired() {
                return Some(json!({"type":"pressed","id":rid,"ok":false,"error":"not-paired"}));
            }
            let (target, hold_key) = if ty == "radio" {
                let what = v.get("what").and_then(Value::as_str).unwrap_or("");
                (PressTarget::Radio(what), format!("radio:{what}"))
            } else {
                let b = v.get("button").and_then(Value::as_str).unwrap_or("");
                (PressTarget::Button(b), b.to_string())
            };
            let keys = match resolve(sh, profile, target) {
                Ok(k) => k.0,
                Err(e) => return Some(json!({"type":"pressed","id":rid,"ok":false,"error":e})),
            };
            let tx = sh.key_tx.lock().unwrap().clone();
            let op = match v.get("down").and_then(Value::as_bool) {
                Some(true) => {
                    if held.contains_key(&hold_key) {
                        return None;
                    }
                    held.insert(hold_key, keys.clone());
                    KeyOp::Down(keys)
                }
                Some(false) => match held.remove(&hold_key) {
                    Some(k) => KeyOp::Up(k),
                    None => return None,
                },
                None => KeyOp::Tap(keys),
            };
            let _ = tx.send(op);
            if let Some(t) = sh.tablets.lock().unwrap().iter_mut().find(|t| t.info.id == id) {
                t.info.presses += 1;
            }
            // Keyboard trouble is reported right away (the press itself runs on the key thread).
            let err = sh.keyboard.error();
            Some(json!({"type":"pressed","id":rid,"ok":err.is_none(),"error":err}))
        }
        "ping" => Some(json!({"type":"pong","t":v.get("t")})),
        _ => None,
    }
}

/// /feed: a community mod pushes {"type":"dash","game":"...", ...} messages.
fn feed_session(sh: Arc<Shared>, mut ws: Ws, peer: SocketAddr) {
    sh.feeds_ws.fetch_add(1, Ordering::Relaxed);
    let source = format!("feed:{}", peer);
    let _ = ws.send(tungstenite::Message::text(json!({"type":"welcome","app":crate::APP_NAME,"version":env!("CARGO_PKG_VERSION")}).to_string()));
    ws.get_ref().set_read_timeout(Some(Duration::from_millis(500))).ok();
    loop {
        if sh.stop.load(Ordering::Relaxed) {
            break;
        }
        match ws.read() {
            Ok(tungstenite::Message::Text(t)) => {
                if let Err(e) = sources::accept_feed(&sh.hub, &source, t.as_str()) {
                    if ws.send(tungstenite::Message::text(json!({"type":"error","message":e}).to_string())).is_err() {
                        break;
                    }
                }
            }
            Ok(tungstenite::Message::Close(_)) => break,
            Ok(_) => {}
            Err(e) if is_timeout(&e) => {}
            Err(_) => break,
        }
    }
    sh.feeds_ws.fetch_sub(1, Ordering::Relaxed);
}

// ---------------------------------------------------------------- helpers

/// The address other devices on the network reach this PC at (the default route's).
pub fn lan_ip() -> Option<String> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.168.0.1:9").or_else(|_| s.connect("10.0.0.1:9")).ok()?;
    let ip = s.local_addr().ok()?.ip();
    if ip.is_unspecified() || ip.is_loopback() { None } else { Some(ip.to_string()) }
}

fn hostname() -> String {
    let h = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .unwrap_or_else(|| "localhost".into());
    if cfg!(windows) || h.contains('.') { h } else { format!("{h}.local") }
}

fn qr_svg(text: &str) -> String {
    use qrcode::render::svg;
    match qrcode::QrCode::new(text.as_bytes()) {
        Ok(q) => q.render::<svg::Color>().min_dimensions(200, 200).quiet_zone(true).dark_color(svg::Color("#000000")).light_color(svg::Color("#ffffff")).build(),
        Err(_) => String::new(),
    }
}

// ---------------------------------------------------------------- game setup

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupInfo {
    pub id: String,
    /// "ready", "missing" (can be set up), "absent" (game not found), "manual".
    pub state: String,
    pub detail: String,
    pub path: Option<String>,
}

fn steam_libraries() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        for v in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(p) = std::env::var_os(v) {
                roots.push(PathBuf::from(p).join("Steam"));
            }
        }
    } else if let Some(h) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(h.join(".local/share/Steam"));
        roots.push(h.join(".steam/steam"));
        roots.push(h.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
    }
    let mut libs: Vec<PathBuf> = Vec::new();
    for r in roots {
        let vdf = r.join("steamapps/libraryfolders.vdf");
        if let Ok(t) = std::fs::read_to_string(&vdf) {
            for line in t.lines() {
                let l = line.trim();
                if let Some(rest) = l.strip_prefix("\"path\"") {
                    let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
                    libs.push(PathBuf::from(p));
                }
            }
        }
        libs.push(r);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for l in libs {
        let c = std::fs::canonicalize(&l).unwrap_or(l);
        if c.is_dir() && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

fn find_game(dir: &str) -> Option<PathBuf> {
    steam_libraries().into_iter().map(|l| l.join("steamapps/common").join(dir)).find(|p| p.is_dir())
}

/// Truck games: the plugin goes into bin/<platform>/plugins. On Linux both the native and the
/// Proton build may exist; we install for whichever is there (both if both).
fn truck_plugin_targets(game_dir: &std::path::Path) -> Vec<(PathBuf, &'static str)> {
    let mut v = Vec::new();
    if game_dir.join("bin/linux_x64").is_dir() {
        v.push((game_dir.join("bin/linux_x64/plugins/v987_dash.so"), "v987_dash.so"));
    }
    if game_dir.join("bin/win_x64").is_dir() {
        v.push((game_dir.join("bin/win_x64/plugins/v987_dash.dll"), "v987_dash.dll"));
    }
    v
}

fn plugin_bytes(name: &str) -> Option<&'static [u8]> {
    ets2_bin::ETS2_PLUGIN.iter().find(|(f, _)| *f == name).map(|(_, b)| *b).filter(|b| !b.is_empty())
}

fn dr2_config_path() -> Option<PathBuf> {
    let rel = "My Games/DiRT Rally 2.0/hardwaresettings/hardware_settings_config.xml";
    if cfg!(windows) {
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from)?;
        Some(home.join("Documents").join(rel))
    } else {
        steam_libraries()
            .into_iter()
            .map(|l| l.join("steamapps/compatdata/690790/pfx/drive_c/users/steamuser/Documents").join(rel))
            .find(|p| p.parent().is_some_and(|d| d.is_dir()))
    }
}

pub fn setup_status() -> Vec<SetupInfo> {
    let mut out = Vec::new();
    // FS22: the mod is its own project.
    let fs22 = profiles::fs22_profile_dir();
    let mod_zip = fs22.as_ref().map(|d| d.join("mods/FS22_WheelFFB.zip")).filter(|p| p.exists());
    let mod_dir = fs22.as_ref().map(|d| d.join("mods/FS22_WheelFFB")).filter(|p| p.is_dir());
    out.push(SetupInfo {
        id: "fs22".into(),
        state: if fs22.is_none() { "absent" } else if mod_zip.is_some() || mod_dir.is_some() { "ready" } else { "manual" }.into(),
        detail: "FS22_WheelFFB".into(),
        path: mod_zip.or(mod_dir).map(|p| p.display().to_string()),
    });
    for (id, dir) in [("ets2", "Euro Truck Simulator 2"), ("ats", "American Truck Simulator")] {
        match find_game(dir) {
            None => out.push(SetupInfo { id: id.into(), state: "absent".into(), detail: String::new(), path: None }),
            Some(g) => {
                let t = truck_plugin_targets(&g);
                let ready = !t.is_empty() && t.iter().all(|(p, f)| std::fs::read(p).ok().as_deref() == plugin_bytes(f));
                out.push(SetupInfo {
                    id: id.into(),
                    state: if ready { "ready" } else { "missing" }.into(),
                    detail: "plugin".into(),
                    path: t.first().map(|(p, _)| p.display().to_string()),
                });
            }
        }
    }
    out.push(SetupInfo {
        id: "ac".into(),
        state: if find_game("assettocorsa").is_some() { "ready" } else { "absent" }.into(),
        detail: "udp 9996".into(),
        path: None,
    });
    match (find_game("DiRT Rally 2.0"), dr2_config_path()) {
        (None, None) => out.push(SetupInfo { id: "dr2".into(), state: "absent".into(), detail: String::new(), path: None }),
        (_, p) => {
            let text = p.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
            let ready = dr2_udp_line(&text).is_some_and(|l| l.contains("enabled=\"true\"") && l.contains("extradata=\"3\"") && l.contains(&format!("port=\"{}\"", sources::DR2_PORT)));
            out.push(SetupInfo {
                id: "dr2".into(),
                state: if ready { "ready" } else if p.as_ref().is_some_and(|p| p.exists()) { "missing" } else { "manual" }.into(),
                detail: "udp 20777".into(),
                path: p.map(|p| p.display().to_string()),
            });
        }
    }
    out
}

fn dr2_udp_line(xml: &str) -> Option<&str> {
    let i = xml.find("<udp ")?;
    let end = xml[i..].find("/>")? + i + 2;
    Some(&xml[i..end])
}

/// Installs the truck plugin or switches on DiRT Rally 2.0's UDP output.
pub fn setup_game(id: &str) -> Result<String, String> {
    match id {
        "ets2" | "ats" => {
            let dir = if id == "ets2" { "Euro Truck Simulator 2" } else { "American Truck Simulator" };
            let g = find_game(dir).ok_or("game not found in your Steam libraries")?;
            let targets = truck_plugin_targets(&g);
            if targets.is_empty() {
                return Err("no bin/linux_x64 or bin/win_x64 folder in the game".into());
            }
            let mut done = Vec::new();
            for (path, file) in targets {
                let bytes = plugin_bytes(file).ok_or("this build of Wheel Studio has no plugin inside; build plugins/ets2-dash first")?;
                std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
                std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                done.push(path.display().to_string());
            }
            Ok(done.join("\n"))
        }
        "dr2" => {
            let p = dr2_config_path().ok_or("DiRT Rally 2.0 settings not found. Start the game once first.")?;
            let xml = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}. Start the game once first.", p.display()))?;
            let want = format!("<udp enabled=\"true\" extradata=\"3\" ip=\"127.0.0.1\" port=\"{}\" delay=\"1\" />", sources::DR2_PORT);
            let new = match dr2_udp_line(&xml) {
                Some(line) => xml.replacen(line, &want, 1),
                None => match xml.find("<motion_platform>") {
                    Some(i) => {
                        let at = i + "<motion_platform>".len();
                        format!("{}\n\t\t{want}{}", &xml[..at], &xml[at..])
                    }
                    None => return Err("no <motion_platform> section in hardware_settings_config.xml".into()),
                },
            };
            let bak = p.with_extension("xml.bak-v987");
            if !bak.exists() {
                std::fs::copy(&p, &bak).map_err(|e| e.to_string())?;
            }
            std::fs::write(&p, new).map_err(|e| e.to_string())?;
            Ok(p.display().to_string())
        }
        _ => Err(format!("nothing to set up for {id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dr2_xml_edit_line() {
        let xml = "<hardware_settings_config>\n<motion_platform>\n<dbox enabled=\"true\" />\n<udp enabled=\"false\" extradata=\"0\" ip=\"127.0.0.1\" port=\"20777\" delay=\"1\" />\n</motion_platform>";
        assert!(dr2_udp_line(xml).unwrap().contains("enabled=\"false\""));
    }

    #[test]
    fn devices() {
        assert_eq!(describe_device("Mozilla/5.0 (Linux; Android 13; SM-X200) AppleWebKit/537.36 Chrome/120 Safari/537.36"), "Android tablet · Chrome");
        assert_eq!(describe_device("Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit Version/17.0 Mobile Safari/604.1"), "iPad · Safari");
    }
}
