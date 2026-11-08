//! Dashboard profiles: one TOML file per game, filling the tablet's single template
//! (radio, readouts, button pages). Built-in ones live in src-tauri/dash-profiles/; files in
//! the user's dash folder with the same `id` replace them. See docs/DASH.md.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

mod bundled {
    include!(concat!(env!("OUT_DIR"), "/dash_profiles.rs"));
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    /// Matched (case-insensitive, substring) against the game id/name the telemetry reports.
    #[serde(rename = "match")]
    pub matches: Vec<String>,
    /// Visual style: "tractor", "truck", "race", "rally" or "default".
    pub theme: String,
    pub accent: String,
    /// Reads key bindings from the game's own settings when a button names an `action`.
    /// Supported: "fs22".
    pub bindings: String,
    /// "Set up" help shown in Wheel Studio and on the tablet's waiting screen.
    pub setup: HashMap<String, String>,
    pub radio: Radio,
    #[serde(rename = "readout")]
    pub readouts: Vec<Readout>,
    #[serde(rename = "page")]
    pub pages: Vec<Page>,
    /// "bundled" or "user"; filled in when loading.
    pub origin: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Radio {
    /// "game" = the radio buttons press the game's radio keys; "tablet" = the tablet plays
    /// internet radio itself. The tablet can switch between them when `power`/`next` exist.
    pub mode: String,
    /// Where the station names on the display come from: "fs22" (the game's station list,
    /// followed as you press next/prev), or empty for a plain tuner display.
    pub stations: String,
    pub power: Option<KeyRef>,
    pub next: Option<KeyRef>,
    pub prev: Option<KeyRef>,
    pub vol_up: Option<KeyRef>,
    pub vol_down: Option<KeyRef>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyRef {
    pub key: String,
    pub action: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Readout {
    pub field: String,
    pub label: String,
    pub labels: HashMap<String, String>,
    pub unit: String,
    pub decimals: u8,
    /// Multiply the raw value (e.g. 100 for 0..1 -> %).
    pub scale: Option<f64>,
    /// "laptime" shows seconds as m:ss.mmm; "text" shows the value as it is.
    pub format: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Page {
    pub name: String,
    pub names: HashMap<String, String>,
    #[serde(rename = "button")]
    pub buttons: Vec<Button>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Button {
    pub id: String,
    pub label: String,
    pub labels: HashMap<String, String>,
    /// Icon name from the tablet's icon set (see docs/DASH.md), or empty for text only.
    pub icon: String,
    /// Key or combo to press ("lctrl+f").
    pub key: String,
    /// Game action whose key is looked up in the game's own bindings first (see `bindings`).
    pub action: String,
    /// Keep the key down while the button is held (horns, look back).
    pub hold: bool,
    /// Telemetry field that lights the button up while true.
    pub active: String,
    /// "warn" (amber), "danger" (red), "ok" (green) or empty.
    pub tone: String,
    /// Grid width in cells (1 or 2).
    pub wide: bool,
}

pub struct Listing {
    pub profiles: Vec<Profile>,
    pub errors: Vec<String>,
}

pub fn user_dir() -> PathBuf {
    ffb_server::config::path().parent().map(Path::to_path_buf).unwrap_or_default().join("dash")
}

pub fn load() -> Listing {
    let mut errors = Vec::new();
    let mut map: Vec<Profile> = Vec::new();
    for (file, text) in bundled::DASH_PROFILES {
        match toml::from_str::<Profile>(text) {
            Ok(mut p) => {
                p.origin = "bundled".into();
                map.push(p);
            }
            Err(e) => errors.push(format!("{file}: {e}")),
        }
    }
    if let Ok(rd) = std::fs::read_dir(user_dir()) {
        let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "toml")).collect();
        files.sort();
        for f in files {
            let name = f.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name == "stations.toml" {
                continue;
            }
            match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|t| toml::from_str::<Profile>(&t).map_err(|e| e.to_string())) {
                Ok(mut p) => {
                    if p.id.is_empty() {
                        p.id = name.trim_end_matches(".toml").to_string();
                    }
                    p.origin = "user".into();
                    map.retain(|x| x.id != p.id);
                    map.push(p);
                }
                Err(e) => errors.push(format!("{name}: {e}")),
            }
        }
    }
    for p in &mut map {
        if p.name.is_empty() {
            p.name = p.id.clone();
        }
        // Make every button addressable.
        for (pi, page) in p.pages.iter_mut().enumerate() {
            for (bi, b) in page.buttons.iter_mut().enumerate() {
                if b.id.is_empty() {
                    b.id = format!("p{pi}b{bi}");
                }
            }
        }
    }
    map.sort_by_key(|p| if p.id == "generic" { 1 } else { 0 });
    Listing { profiles: map, errors }
}

/// Best profile for a game id/name reported by telemetry.
pub fn pick<'a>(profiles: &'a [Profile], game: &str) -> Option<&'a Profile> {
    let g = game.to_lowercase();
    if g.is_empty() {
        return None;
    }
    profiles
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(&g))
        .or_else(|| profiles.iter().find(|p| p.matches.iter().any(|m| !m.is_empty() && g.contains(&m.to_lowercase()))))
}

// ---------------------------------------------------------------- game bindings

/// Reads key bindings from a game's settings, cached by modification time.
#[derive(Default)]
pub struct Bindings {
    cache: HashMap<String, (PathBuf, SystemTime, HashMap<String, String>)>,
}

impl Bindings {
    /// The keyboard combo bound to `action` in the game's own naming ("KEY_lctrl KEY_f" for
    /// FS22, "lctrl+lbracket" for SCS games). `kind`: "fs22", "ets2" or "ats".
    pub fn lookup(&mut self, kind: &str, action: &str) -> Option<String> {
        if action.is_empty() {
            return None;
        }
        let path = match kind {
            "fs22" => fs22_profile_dir()?.join("inputBinding.xml"),
            "ets2" => scs_controls_file("Euro Truck Simulator 2")?,
            "ats" => scs_controls_file("American Truck Simulator")?,
            _ => return None,
        };
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok()?;
        let fresh = matches!(self.cache.get(kind), Some((p, t, _)) if *p == path && *t == mtime);
        if !fresh {
            let text = std::fs::read_to_string(&path).ok()?;
            let map = if kind == "fs22" { parse_fs22_bindings(&text) } else { parse_scs_controls(&text) };
            self.cache.insert(kind.into(), (path, mtime, map));
        }
        self.cache.get(kind).and_then(|(_, _, m)| m.get(action).cloned())
    }
}

/// The controls file of the most recently used ETS2/ATS profile.
pub fn scs_controls_file(game: &str) -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("USERPROFILE").map(PathBuf::from)?.join("Documents").join(game)
    } else {
        std::env::var_os("HOME").map(PathBuf::from)?.join(".local/share").join(game)
    };
    let names: &[&str] = if cfg!(windows) { &["controls.sii"] } else { &["controls_linux.sii", "controls.sii"] };
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for sub in ["steam_profiles", "profiles"] {
        let Ok(rd) = std::fs::read_dir(base.join(sub)) else { continue };
        for e in rd.flatten() {
            for n in names {
                let f = e.path().join(n);
                if let Ok(t) = std::fs::metadata(&f).and_then(|m| m.modified()) {
                    if best.as_ref().is_none_or(|(bt, _)| t > *bt) {
                        best = Some((t, f));
                    }
                    break;
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// `config_lines[n]: "mix light `keyboard.l?0 | joy.b2?0`"` -> {"light": "l"}; modifiers
/// become combos ("lctrl+f9").
pub fn parse_scs_controls(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let Some(i) = line.find("mix ") else { continue };
        let rest = &line[i + 4..];
        let Some(sp) = rest.find(' ') else { continue };
        let name = &rest[..sp];
        let Some(a) = rest.find('`') else { continue };
        let Some(b) = rest[a + 1..].find('`') else { continue };
        let expr = &rest[a + 1..a + 1 + b];
        for term in expr.split('|').map(str::trim).filter(|t| !t.is_empty()) {
            if term.contains('&') || !term.contains("keyboard.") {
                continue;
            }
            let k = term.split("keyboard.").nth(1).and_then(|x| x.split('?').next()).unwrap_or("");
            if k.is_empty() {
                continue;
            }
            let modifier = term.strip_prefix("modifier(").and_then(|m| m.split(',').next()).unwrap_or("no_modifier").trim();
            let pre = match modifier {
                "ctrl_only" => "lctrl+",
                "shift_only" => "lshift+",
                "alt_only" => "lalt+",
                "ctr_shf" => "lctrl+lshift+",
                "no_modifier" => "",
                _ => continue,
            };
            out.insert(name.to_string(), format!("{pre}{k}"));
            break;
        }
    }
    out
}

/// First keyboard binding per action from FS22's inputBinding.xml.
pub fn parse_fs22_bindings(xml: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<actionBinding action=\"") {
        rest = &rest[i + 23..];
        let Some(q) = rest.find('"') else { break };
        let action = rest[..q].to_string();
        let end = rest.find("</actionBinding>").unwrap_or(rest.len());
        let body = &rest[..end];
        for line in body.split("<binding ").skip(1) {
            if !line.contains("device=\"KB_MOUSE_DEFAULT\"") {
                continue;
            }
            if let Some(s) = line.find("input=\"") {
                let v = &line[s + 7..];
                if let Some(e) = v.find('"') {
                    let input = &v[..e];
                    if input.starts_with("KEY_") && !input.contains("MOUSE") {
                        out.entry(action.clone()).or_insert_with(|| input.to_string());
                        break;
                    }
                }
            }
        }
        rest = &rest[end..];
    }
    out
}

/// FS22's "My Games" folder: in the Proton prefix on Linux, Documents on Windows.
pub fn fs22_profile_dir() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = if cfg!(windows) {
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from)?;
        vec![home.join("Documents/My Games/FarmingSimulator2022"), home.join("OneDrive/Documents/My Games/FarmingSimulator2022")]
    } else {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let tail = "steamapps/compatdata/1248130/pfx/drive_c/users/steamuser/Documents/My Games/FarmingSimulator2022";
        vec![
            home.join(".local/share/Steam").join(tail),
            home.join(".steam/steam").join(tail),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam").join(tail),
        ]
    };
    candidates.into_iter().find(|p| p.is_dir())
}

// ---------------------------------------------------------------- radio stations

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Station {
    pub name: String,
    pub url: String,
}

#[derive(Default, Deserialize)]
struct StationsFile {
    #[serde(default, rename = "station")]
    stations: Vec<Station>,
}

/// Stations for the tablet radio: the user's stations.toml, then FS22's own list (so the
/// tablet and the game agree), then a few defaults.
pub fn stations() -> (Vec<Station>, Vec<Station>) {
    let fs22 = fs22_stations();
    let mut list: Vec<Station> = std::fs::read_to_string(user_dir().join("stations.toml"))
        .ok()
        .and_then(|t| toml::from_str::<StationsFile>(&t).ok())
        .map(|f| f.stations)
        .unwrap_or_default();
    for s in &fs22 {
        if !list.iter().any(|x| x.url == s.url) {
            list.push(s.clone());
        }
    }
    if list.is_empty() {
        list = vec![
            Station { name: "Radio Paradise".into(), url: "https://stream.radioparadise.com/mp3-128".into() },
            Station { name: "SomaFM Groove Salad".into(), url: "https://ice1.somafm.com/groovesalad-128-mp3".into() },
        ];
    }
    (list, fs22)
}

/// FS22's station file has only URLs (the game shows the stream's own name). We use the
/// name the stream announces (icy-name, looked up in the background and cached), or one
/// guessed from the URL.
fn fs22_stations() -> Vec<Station> {
    let Some(dir) = fs22_profile_dir() else { return vec![] };
    let Ok(xml) = std::fs::read_to_string(dir.join("music/streamingInternetRadios.xml")) else { return vec![] };
    let cache = name_cache();
    let mut out = Vec::new();
    for (i, tag) in xml.split("<streamingInternetRadio ").skip(1).enumerate() {
        let attr = |k: &str| {
            let pat = format!("{k}=\"");
            tag.find(&pat).and_then(|i| {
                let v = &tag[i + pat.len()..];
                v.find('"').map(|e| xml_unescape(&v[..e]))
            })
        };
        if let Some(url) = attr("href") {
            let name = attr("name")
                .filter(|n| !n.trim().is_empty())
                .or_else(|| cache.get(&url).cloned())
                .unwrap_or_else(|| guess_station_name(&url, i + 1));
            out.push(Station { name, url });
        }
    }
    out
}

fn name_cache_path() -> PathBuf {
    user_dir().join(".station-names.json")
}

fn name_cache() -> HashMap<String, String> {
    std::fs::read_to_string(name_cache_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// "https://listen.x.com.tr/powerfm/mpeg/icecast.audio" -> "Power FM"; anonymous streams
/// ("…:10990/stream") -> "Radio 6".
pub fn guess_station_name(url: &str, n: usize) -> String {
    const NOISE: &[&str] = &[
        "stream", "streams", "live", "listen", "mpeg", "icecast", "audio", "api", "livestream", "redirect", "mp3", "aac", "sc",
        "proxy", "radio", "http", "https", "www", "com", "tr", "org", "net", "1", "type", "nocache", "mp", "high", "low", "hq",
    ];
    let no_query = url.split(['?', '#']).next().unwrap_or(url);
    let path = no_query.splitn(4, '/').nth(3).unwrap_or("");
    let mut best = String::new();
    for seg in path.split('/') {
        let seg = seg.trim_start_matches(';').split('.').next().unwrap_or("");
        let words: Vec<&str> = seg
            .split(['_', '-', ' '])
            .filter(|w| !w.is_empty() && !NOISE.contains(&w.to_ascii_lowercase().as_str()) && !w.chars().all(|c| c.is_ascii_digit() || c == '*'))
            .collect();
        if !words.is_empty() && words.iter().any(|w| w.chars().any(|c| c.is_ascii_alphabetic())) {
            best = words.join(" ");
        }
    }
    if best.is_empty() || best.len() < 3 {
        return format!("Radio {n}");
    }
    let pretty: Vec<String> = best
        .split(' ')
        .map(|w| {
            let l = w.to_ascii_lowercase();
            if l == "fm" {
                return "FM".to_string();
            }
            // "tgrtfm" -> "TGRT FM", "powerfm" -> "Power FM"
            if l.len() > 3 && l.ends_with("fm") {
                let head = &l[..l.len() - 2];
                let head = if head.len() <= 4 { head.to_uppercase() } else { cap(head) };
                return format!("{head} FM");
            }
            cap(&l)
        })
        .collect();
    pretty.join(" ")
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Asks each unnamed FS22 stream for its icy-name and caches the answers. Runs once in the
/// background when the dashboard starts.
pub fn learn_station_names() {
    let Some(dir) = fs22_profile_dir() else { return };
    let Ok(xml) = std::fs::read_to_string(dir.join("music/streamingInternetRadios.xml")) else { return };
    let mut cache = name_cache();
    let urls: Vec<String> = xml
        .split("href=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next().map(xml_unescape))
        .filter(|u| !cache.contains_key(u))
        .collect();
    if urls.is_empty() {
        return;
    }
    let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() else { return };
    let found: Vec<(String, String)> = rt.block_on(async {
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(8)).build().ok()?;
        let mut out = Vec::new();
        for u in urls {
            if let Ok(r) = client.get(&u).header("Icy-MetaData", "1").send().await {
                if let Some(n) = r.headers().get("icy-name").and_then(|v| v.to_str().ok()) {
                    let n = n.trim();
                    if !n.is_empty() && !n.eq_ignore_ascii_case("no name") && n.len() < 80 {
                        out.push((u.clone(), n.to_string()));
                    }
                }
                drop(r); // don't download the stream
            }
        }
        Some(out)
    })
    .unwrap_or_default();
    if found.is_empty() {
        return;
    }
    cache.extend(found);
    let _ = std::fs::create_dir_all(user_dir());
    let _ = std::fs::write(name_cache_path(), serde_json::to_string_pretty(&cache).unwrap_or_default());
}

fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&").replace("&quot;", "\"").replace("&apos;", "'").replace("&lt;", "<").replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_profiles_parse_and_keys_resolve() {
        let l = load();
        assert!(l.errors.is_empty(), "{:?}", l.errors);
        for id in ["fs22", "ets2", "ats", "ac", "dr2", "generic"] {
            assert!(l.profiles.iter().any(|p| p.id == id), "missing {id}");
        }
        for p in &l.profiles {
            for b in p.pages.iter().flat_map(|pg| &pg.buttons) {
                assert!(!b.key.is_empty() || !b.action.is_empty(), "{}: {} has no key", p.id, b.id);
                if !b.key.is_empty() {
                    super::super::keys::parse_combo(&b.key).unwrap_or_else(|e| panic!("{}: {}: {e}", p.id, b.id));
                }
            }
            for k in [&p.radio.power, &p.radio.next, &p.radio.prev, &p.radio.vol_up, &p.radio.vol_down].into_iter().flatten() {
                if !k.key.is_empty() {
                    super::super::keys::parse_combo(&k.key).unwrap();
                }
            }
        }
        assert_eq!(pick(&l.profiles, "Farming Simulator 22").unwrap().id, "fs22");
        assert_eq!(pick(&l.profiles, "dr2").unwrap().id, "dr2");
    }

    #[test]
    fn station_names_from_urls() {
        let g = |u| guess_station_name(u, 6);
        assert_eq!(g("https://listen.powerapp.com.tr/powerfm/mpeg/icecast.audio"), "Power FM");
        assert_eq!(g("https://playerservices.streamtheworld.com/api/livestream-redirect/SUPER_FM.mp3"), "Super FM");
        assert_eq!(g("https://27743.live.streamtheworld.com/JOY_TURK_SC"), "Joy Turk");
        assert_eq!(g("https://playerservices.streamtheworld.com/api/livestream-redirect/JOYTURK_ROCK_SC?/"), "Joyturk Rock");
        assert_eq!(g("https://icecasttgrt.ihlasdigitalassets.com/tgrtfm"), "TGRT FM");
        assert_eq!(g("https://yayin2.canliyayin.org:10990/;*.mp3"), "Radio 6");
        assert_eq!(g("https://server7.streamserver24.com/proxy/turkiyemfm?mp=/stream"), "Turkiyem FM");
        assert_eq!(g("https://anadolu.liderhost.com.tr:10980/stream"), "Radio 6");
    }

    #[test]
    fn fs22_binding_parse() {
        let xml = r#"<actionBinding action="TOGGLE_LIGHTS">
        <binding device="0_XINPUT_GAMEPAD" input="BUTTON_3" index="1"/>
        <binding device="KB_MOUSE_DEFAULT" input="KEY_f" axisComponent="+" neutralInput="0" index="1"/>
    </actionBinding>
    <actionBinding action="HONK">
        <binding device="KB_MOUSE_DEFAULT" input="MOUSE_BUTTON_LEFT" index="2"/>
        <binding device="KB_MOUSE_DEFAULT" input="KEY_0" index="1"/>
    </actionBinding>
    <actionBinding action="NONE"></actionBinding>"#;
        let m = parse_fs22_bindings(xml);
        assert_eq!(m["TOGGLE_LIGHTS"], "KEY_f");
        assert_eq!(m["HONK"], "KEY_0");
        assert!(!m.contains_key("NONE"));
    }

    #[test]
    fn scs_controls_parse() {
        let t = r#" config_lines[120]: "mix light `keyboard.l?0 | short_press(joy.b2?0) | semantical.light?0`"
 config_lines[121]: "mix lblinker `keyboard.lbracket?0 | joy.b5?0`"
 config_lines[122]: "mix teleport `modifier(ctrl_only, keyboard.f9?0)`"
 config_lines[123]: "mix sndedittool `keyboard.lshift?0 & keyboard.rshift?0`"
 config_lines[124]: "mix quicksave `keyboard.scrollock?0 | semantical.quicksave?0`"
 config_lines[125]: "mix cam1 `keyboard.key1?0 | long_press(joy.pov1_up?0)`""#;
        let m = parse_scs_controls(t);
        assert_eq!(m["light"], "l");
        assert_eq!(m["lblinker"], "lbracket");
        assert_eq!(m["teleport"], "lctrl+f9");
        assert!(!m.contains_key("sndedittool"));
        for v in m.values() {
            super::super::keys::parse_combo(v).unwrap();
        }
    }
}
