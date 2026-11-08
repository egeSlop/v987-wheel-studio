mod dash;
mod link;
mod presets;
mod protocol;

use std::sync::Mutex;

use ffb_server::FfbServer;

use protocol::{Curve, DeviceState, Profile};
use serde::Serialize;
use tauri::{Manager, State};

use link::{FoundDevice, Link};

type Res<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectInfo {
    name: String,
    state: Option<DeviceState>,
}

/// Outcome of a write: whether the wheel echoed it, and what it reports afterwards.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Applied {
    acked: bool,
    ack: Option<String>,
    /// Fresh read of the profile page(s) the write touched.
    readback: Option<Profile>,
}

#[tauri::command]
async fn scan(link: State<'_, Link>, millis: Option<u64>) -> Res<Vec<FoundDevice>> {
    link.scan(millis.unwrap_or(4000)).await
}

#[tauri::command]
async fn connect(link: State<'_, Link>, id: String) -> Res<ConnectInfo> {
    let name = link.connect(&id).await?;
    let state = read_state_inner(&link).await?;
    Ok(ConnectInfo { name, state })
}

#[tauri::command]
async fn disconnect(link: State<'_, Link>) -> Res<()> {
    link.disconnect().await;
    Ok(())
}

#[tauri::command]
async fn connected(link: State<'_, Link>) -> Res<Option<String>> {
    Ok(link.connected_name().await)
}

async fn read_state_inner(link: &Link) -> Res<Option<DeviceState>> {
    let r = link.request(&[protocol::READ_STATE, 1], 3).await?;
    Ok(r.and_then(|d| protocol::decode_state(&d)))
}

#[tauri::command]
async fn read_state(link: State<'_, Link>) -> Res<Option<DeviceState>> {
    read_state_inner(&link).await
}

async fn read_pages(link: &Link, mode: u8, pages: &[u8]) -> Res<Profile> {
    let mut p = Profile { mode, ..Default::default() };
    for &page in pages {
        if let Some(d) = link.request(&protocol::read_page_request(mode, page), 3).await? {
            protocol::apply_page(&mut p, &d);
        }
    }
    Ok(p)
}

#[tauri::command]
async fn read_profile(link: State<'_, Link>, mode: u8) -> Res<Profile> {
    read_pages(&link, mode, &[0, 1, 2, 3, 4, 5]).await
}

/// Sends an edit, checks the echo, then reads back `pages`.
async fn apply(link: &Link, req: Vec<u8>, mode: u8, pages: &[u8]) -> Res<Applied> {
    let reply = link.request(&req, 3).await?;
    let acked = reply
        .as_ref()
        .is_some_and(|d| d.first() == Some(&protocol::response_id(req[0])) && d.get(1) == req.get(1));
    let readback = if pages.is_empty() { None } else { Some(read_pages(link, mode, pages).await?) };
    Ok(Applied { acked, ack: reply.map(|d| protocol::hex(&d)), readback })
}

#[tauri::command]
async fn set_angle(link: State<'_, Link>, mode: u8, angle: u16) -> Res<Applied> {
    apply(&link, protocol::angle_request(angle), mode, &[2]).await
}

#[tauri::command]
async fn set_ffb(link: State<'_, Link>, mode: u8, ffb: u8, reverse: bool) -> Res<Applied> {
    apply(&link, protocol::ffb_request(mode, ffb, reverse), mode, &[2]).await
}

#[tauri::command]
async fn set_forces(
    link: State<'_, Link>,
    mode: u8,
    spring: u8,
    damping: u8,
    friction: u8,
    inertia: u8,
) -> Res<Applied> {
    apply(&link, protocol::forces_request(spring, damping, friction, inertia), mode, &[2]).await
}

#[tauri::command]
async fn set_keymap(link: State<'_, Link>, mode: u8, target: u8, replacement: u8) -> Res<Applied> {
    apply(&link, protocol::keymap_request(mode, target, replacement), mode, &[0, 1, 2]).await
}

#[tauri::command]
async fn write_curve(link: State<'_, Link>, mode: u8, pedal: u8, curve: Curve) -> Res<Applied> {
    if pedal > 2 {
        return Err("pedal must be 0 (throttle), 1 (brake) or 2 (clutch)".into());
    }
    apply(&link, protocol::write_curve_request(mode, pedal, &curve), mode, &[3 + pedal]).await
}

#[tauri::command]
async fn set_mode(link: State<'_, Link>, mode: u8) -> Res<Applied> {
    apply(&link, protocol::mode_request(mode), mode, &[]).await
}

#[tauri::command]
async fn reset_profile(link: State<'_, Link>, mode: u8) -> Res<Applied> {
    apply(&link, protocol::reset_request(mode), mode, &[0, 1, 2, 3, 4, 5]).await
}

#[tauri::command]
async fn test_mode(link: State<'_, Link>, on: bool) -> Res<bool> {
    if on {
        Ok(link.request(&protocol::test_request(true), 2).await?.is_some())
    } else {
        link.send(&protocol::test_request(false)).await?;
        Ok(true)
    }
}

/// Raw request for the Labs panel. Returns the reply hex, or None if the wheel stayed silent.
#[tauri::command]
async fn raw(link: State<'_, Link>, bytes: Vec<u8>) -> Res<Option<String>> {
    if bytes.is_empty() {
        return Err("empty request".into());
    }
    Ok(link.request(&bytes, 1).await?.map(|d| protocol::hex(&d)))
}

// ---------------------------------------------------------------- presets

#[tauri::command]
fn list_presets() -> presets::Listing {
    presets::list()
}

/// Downloads every vendor preset (PC, PS4, Xbox) into the local preset folder.
#[tauri::command]
async fn refresh_vendor_presets() -> Res<presets::Listing> {
    presets::refresh_vendor().await?;
    Ok(presets::list())
}

#[tauri::command]
fn save_user_preset(preset: presets::Preset) -> Res<presets::Preset> {
    presets::save_user(preset)
}

#[tauri::command]
fn delete_user_preset(id: String) -> Res<()> {
    presets::delete_user(&id)
}

// ---------------------------------------------------------------- FFB server

/// The built-in FFB server. None while stopped.
#[derive(Default)]
struct Ffb {
    server: Mutex<Option<FfbServer>>,
    last_error: Mutex<Option<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FfbState {
    /// This app's server is running.
    running: bool,
    /// Why it isn't (e.g. the port is taken by the headless server).
    error: Option<String>,
    /// Something else answers on the port: watch it through /stream.
    external: bool,
    status: Option<ffb_server::Status>,
    config: ffb_server::Config,
    config_path: String,
}

fn port_answers(listen: &str) -> bool {
    use std::net::ToSocketAddrs;
    listen
        .to_socket_addrs()
        .ok()
        .and_then(|mut a| a.next())
        .is_some_and(|a| std::net::TcpStream::connect_timeout(&a, std::time::Duration::from_millis(150)).is_ok())
}

fn ffb_state_of(ffb: &Ffb) -> FfbState {
    let srv = ffb.server.lock().unwrap();
    let config = srv.as_ref().map(|s| s.config()).unwrap_or_else(ffb_server::config::load);
    let running = srv.is_some();
    FfbState {
        running,
        error: if running { None } else { ffb.last_error.lock().unwrap().clone() },
        external: !running && port_answers(&config.listen),
        status: srv.as_ref().map(|s| s.status()),
        config_path: ffb_server::config::path().display().to_string(),
        config,
    }
}

fn start_ffb(ffb: &Ffb) -> Result<(), String> {
    let mut srv = ffb.server.lock().unwrap();
    if srv.is_some() {
        return Ok(());
    }
    match FfbServer::start(APP_NAME, ffb_server::config::load()) {
        Ok(s) => {
            *srv = Some(s);
            *ffb.last_error.lock().unwrap() = None;
            Ok(())
        }
        Err(e) => {
            let msg = format!("{e:#}");
            *ffb.last_error.lock().unwrap() = Some(msg.clone());
            Err(msg)
        }
    }
}

#[tauri::command]
fn ffb_state(ffb: State<'_, Ffb>) -> FfbState {
    ffb_state_of(&ffb)
}

#[tauri::command]
fn ffb_start(ffb: State<'_, Ffb>) -> Res<FfbState> {
    start_ffb(&ffb)?;
    Ok(ffb_state_of(&ffb))
}

#[tauri::command]
fn ffb_stop(ffb: State<'_, Ffb>) -> FfbState {
    let s = ffb.server.lock().unwrap().take();
    if let Some(s) = s {
        s.stop();
    }
    *ffb.last_error.lock().unwrap() = None;
    ffb_state_of(&ffb)
}

/// Saves settings; a running server (this one or the headless one) picks them up immediately.
#[tauri::command]
fn ffb_set_config(ffb: State<'_, Ffb>, config: ffb_server::Config) -> Res<FfbState> {
    let restart = {
        let srv = ffb.server.lock().unwrap();
        match srv.as_ref() {
            Some(s) => {
                let listen_changed = s.config().listen != config.listen;
                s.set_config(config).map_err(|e| e.to_string())?;
                listen_changed
            }
            None => {
                ffb_server::config::save(&config.sanitized()).map_err(|e| e.to_string())?;
                false
            }
        }
    };
    if restart {
        if let Some(s) = ffb.server.lock().unwrap().take() {
            s.stop();
        }
        let _ = start_ffb(&ffb);
    }
    Ok(ffb_state_of(&ffb))
}

#[tauri::command]
fn ffb_test(ffb: State<'_, Ffb>) -> Res<bool> {
    match ffb.server.lock().unwrap().as_ref() {
        Some(s) => Ok(s.run_test()),
        None => Err("Start the FFB server first.".into()),
    }
}

// ---------------------------------------------------------------- tablet dashboard

#[derive(Default)]
struct Dash {
    server: Mutex<Option<dash::DashServer>>,
    last_error: Mutex<Option<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashState {
    running: bool,
    error: Option<String>,
    config: dash::Config,
    status: Option<dash::Status>,
    setup: Vec<dash::SetupInfo>,
    config_path: String,
}

fn dash_state_of(d: &Dash, with_setup: bool) -> DashState {
    let srv = d.server.lock().unwrap();
    DashState {
        running: srv.is_some(),
        error: if srv.is_some() { None } else { d.last_error.lock().unwrap().clone() },
        config: srv.as_ref().map(|s| s.config()).unwrap_or_else(dash::load_config),
        status: srv.as_ref().map(|s| s.status()),
        setup: if with_setup { dash::setup_status() } else { Vec::new() },
        config_path: dash::config_path().display().to_string(),
    }
}

fn start_dash(d: &Dash) -> Result<(), String> {
    let mut srv = d.server.lock().unwrap();
    if srv.is_some() {
        return Ok(());
    }
    match dash::DashServer::start(dash::load_config()) {
        Ok(s) => {
            // Create the virtual keyboard early so the desktop knows it before the first press.
            let _ = s.warm_keyboard();
            *srv = Some(s);
            *d.last_error.lock().unwrap() = None;
            Ok(())
        }
        Err(e) => {
            let msg = format!("{e:#}");
            *d.last_error.lock().unwrap() = Some(msg.clone());
            Err(msg)
        }
    }
}

#[tauri::command]
fn dash_state(d: State<'_, Dash>, setup: Option<bool>) -> DashState {
    dash_state_of(&d, setup.unwrap_or(false))
}

#[tauri::command]
fn dash_start(d: State<'_, Dash>) -> Res<DashState> {
    start_dash(&d)?;
    Ok(dash_state_of(&d, false))
}

#[tauri::command]
fn dash_stop(d: State<'_, Dash>) -> DashState {
    if let Some(s) = d.server.lock().unwrap().take() {
        s.stop();
    }
    *d.last_error.lock().unwrap() = None;
    dash_state_of(&d, false)
}

#[tauri::command]
fn dash_set_config(d: State<'_, Dash>, config: dash::Config) -> Res<DashState> {
    let restart = {
        let srv = d.server.lock().unwrap();
        match srv.as_ref() {
            Some(s) => s.set_config(config).map_err(|e| e.to_string())?,
            None => {
                dash::save_config(&config).map_err(|e| e.to_string())?;
                false
            }
        }
    };
    if restart {
        if let Some(s) = d.server.lock().unwrap().take() {
            s.stop();
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
        let _ = start_dash(&d);
    }
    Ok(dash_state_of(&d, false))
}

#[tauri::command]
fn dash_new_pin(d: State<'_, Dash>) -> Res<DashState> {
    let mut c = d.server.lock().unwrap().as_ref().map(|s| s.config()).unwrap_or_else(dash::load_config);
    c.pin = dash::new_pin();
    dash_set_config(d, c)
}

#[tauri::command]
async fn dash_setup_game(id: String) -> Res<String> {
    tauri::async_runtime::spawn_blocking(move || dash::setup_game(&id)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn dash_test(d: State<'_, Dash>, profile: String, button: String) -> Res<String> {
    match d.server.lock().unwrap().as_ref() {
        Some(s) => s.test_press(&profile, &button),
        None => Err("Start the tablet server first.".into()),
    }
}

#[tauri::command]
fn dash_reload(d: State<'_, Dash>) -> DashState {
    if let Some(s) = d.server.lock().unwrap().as_ref() {
        s.reload_profiles();
    }
    dash_state_of(&d, false)
}

/// Opens the folder for custom profiles (creating it, with a README, the first time).
#[tauri::command]
fn dash_open_profiles() -> Res<String> {
    let dir = dash::profiles::user_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let readme = dir.join("README.txt");
    if !readme.exists() {
        let _ = std::fs::write(
            &readme,
            "Custom tablet profiles for V987 Wheel Studio.\n\nPut one .toml file per game here. A file with the same id as a built-in profile\n(fs22, ets2, ats, ac, dr2, generic) replaces it. stations.toml adds tablet radio stations.\nFormat: https://github.com/egeSlop/v987-wheel-studio/blob/main/docs/DASH.md\n",
        );
    }
    let opener = if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    std::process::Command::new(opener).arg(&dir).spawn().map_err(|e| e.to_string())?;
    Ok(dir.display().to_string())
}

pub const APP_NAME: &str = "V987 Wheel Studio";

/// `v987-wheel-studio --ffb-server [run|test|config]`: the server without a window,
/// for game launch scripts.
pub fn headless(args: &[&str]) -> anyhow::Result<()> {
    // The tablet dashboard comes along when the server runs (not for `test` or `config`).
    let _dash = if matches!(args.first(), None | Some(&"run")) && dash::load_config().autostart {
        dash::DashServer::start(dash::load_config()).map_err(|e| eprintln!("[dash] not started: {e:#}")).ok()
    } else {
        None
    };
    ffb_server::cli::run(args, APP_NAME)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Ffb::default())
        .manage(Dash::default())
        .setup(|app| {
            app.manage(Link::new(app.handle().clone()));
            if ffb_server::config::load().autostart {
                let ffb = app.state::<Ffb>();
                if let Err(e) = start_ffb(&ffb) {
                    eprintln!("[wheel-studio] FFB server not started: {e}");
                }
            }
            if dash::load_config().autostart {
                let d = app.state::<Dash>();
                if let Err(e) = start_dash(&d) {
                    eprintln!("[wheel-studio] tablet server not started: {e}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan,
            connect,
            disconnect,
            connected,
            read_state,
            read_profile,
            set_angle,
            set_ffb,
            set_forces,
            set_keymap,
            write_curve,
            set_mode,
            reset_profile,
            test_mode,
            raw,
            list_presets,
            refresh_vendor_presets,
            save_user_preset,
            delete_user_preset,
            ffb_state,
            ffb_start,
            ffb_stop,
            ffb_set_config,
            ffb_test,
            dash_state,
            dash_start,
            dash_stop,
            dash_set_config,
            dash_new_pin,
            dash_setup_game,
            dash_test,
            dash_reload,
            dash_open_profiles
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Stop cleanly so the wheel's effects are removed and it returns to its idle spring.
            if let Some(s) = handle.state::<Ffb>().server.lock().unwrap().take() {
                s.stop();
            }
            if let Some(s) = handle.state::<Dash>().server.lock().unwrap().take() {
                s.stop();
            }
        }
    });
}
