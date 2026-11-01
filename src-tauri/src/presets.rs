//! Presets: one TOML file each.
//!
//!   bundled      src-tauri/presets/*.toml, compiled in (vendor snapshot + Wheel Studio's own)
//!   downloaded   ~/.local/share/v987-wheel-studio/presets/vendor/*.toml ("Refresh from vendor")
//!   yours        ~/.local/share/v987-wheel-studio/presets/user/*.toml
//!
//! Later sources override earlier ones with the same id. Format: docs/PRESETS.md.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ffb_server::model::Feel;
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/bundled_presets.rs"));

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WheelParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ffb: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spring: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damping: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub friction: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inertia: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FfbProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_gain: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invert: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feel: Option<Feel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VendorInfo {
    pub id: String,
    pub mode: u8,
    pub raw: String,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    #[serde(default = "one")]
    pub format: u32,
    pub id: String,
    pub name: String,
    /// "pc", "ps4" or "xbox".
    #[serde(default)]
    pub platform: String,
    /// "vendor", "studio" or "user".
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tutorial: Option<String>,
    /// Names in other languages, by code ("zh", "tr").
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub names: BTreeMap<String, String>,
    /// Notes by language code ("en", "tr", "zh").
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub notes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    #[serde(default)]
    pub wheel: WheelParams,
    /// Only remapped controls: code -> replacement.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub keymap: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ffb_server: Option<FfbProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<VendorInfo>,
    /// Where it was loaded from: "bundled", "downloaded" or "user". Not stored.
    #[serde(default, skip_deserializing)]
    pub origin: String,
}

/// ~/.local/share/v987-wheel-studio/presets on Linux, %APPDATA%\v987-wheel-studio\presets on Windows.
pub fn data_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default()
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"))
    };
    base.join("v987-wheel-studio").join("presets")
}

fn read_dir(dir: &PathBuf, origin: &str, out: &mut Vec<Preset>, errors: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "toml")).collect();
    files.sort();
    for f in files {
        match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|s| toml::from_str::<Preset>(&s).map_err(|e| e.to_string())) {
            Ok(mut p) => {
                p.origin = origin.into();
                out.push(p);
            }
            Err(e) => errors.push(format!("{}: {e}", f.display())),
        }
    }
}

fn upsert(list: &mut Vec<Preset>, p: Preset) {
    match list.iter_mut().find(|x| x.id == p.id) {
        Some(old) => {
            // A refreshed vendor file has no translations: keep the bundled ones.
            let mut p = p;
            for (k, v) in &old.notes {
                p.notes.entry(k.clone()).or_insert_with(|| v.clone());
            }
            for (k, v) in &old.names {
                p.names.entry(k.clone()).or_insert_with(|| v.clone());
            }
            if p.links.is_empty() {
                p.links = old.links.clone();
            }
            *old = p;
        }
        None => list.push(p),
    }
}

#[derive(Serialize)]
pub struct Listing {
    pub presets: Vec<Preset>,
    pub errors: Vec<String>,
    pub dir: String,
}

pub fn list() -> Listing {
    let mut all = Vec::new();
    let mut errors = Vec::new();
    for (file, body) in BUNDLED {
        match toml::from_str::<Preset>(body) {
            Ok(mut p) => {
                p.origin = "bundled".into();
                upsert(&mut all, p);
            }
            Err(e) => errors.push(format!("bundled {file}: {e}")),
        }
    }
    for (sub, origin) in [("vendor", "downloaded"), ("user", "user")] {
        let mut found = Vec::new();
        read_dir(&data_dir().join(sub), origin, &mut found, &mut errors);
        for p in found {
            upsert(&mut all, p);
        }
    }
    Listing { presets: all, errors, dir: data_dir().display().to_string() }
}

fn safe_id(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

pub fn save_user(mut p: Preset) -> Result<Preset, String> {
    if p.name.trim().is_empty() {
        return Err("a preset needs a name".into());
    }
    if p.id.trim().is_empty() || !p.id.starts_with("user-") {
        p.id = format!("user-{}", safe_id(&p.name.trim().to_lowercase()));
    }
    p.source = "user".into();
    p.format = 1;
    let dir = data_dir().join("user");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let body = toml::to_string_pretty(&p).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{}.toml", safe_id(&p.id))), body).map_err(|e| e.to_string())?;
    p.origin = "user".into();
    Ok(p)
}

pub fn delete_user(id: &str) -> Result<(), String> {
    if !id.starts_with("user-") {
        return Err("only your own presets can be deleted".into());
    }
    let f = data_dir().join("user").join(format!("{}.toml", safe_id(id)));
    std::fs::remove_file(f).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- vendor blobs

/// Key order of a vendor blob, from the vendor app's DetailActivity (mode byte 0).
const SEQ_V1: [u8; 35] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 17, 16, 47, 14, 15, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];

/// Decodes a vendor `keboardValue`: mode, key map, angle u16 LE, ffb, spring, damping, friction,
/// inertia, [reverse], [pedal curves]. Mode 0 uses SEQ_V1; 3/5/8 repeat code 47 after code 15.
pub fn decode_blob(hex: &str) -> Option<(u8, BTreeMap<String, u8>, WheelParams)> {
    let b: Vec<u8> = (0..hex.len() / 2).map(|i| u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()).collect::<Option<_>>()?;
    let mode = *b.first()?;
    let mut seq: Vec<u8> = SEQ_V1.to_vec();
    if mode != 0 {
        seq.insert(19, 47);
    }
    let o = 1 + seq.len();
    if b.len() < o + 7 {
        return None;
    }
    let mut keymap = BTreeMap::new();
    for (i, code) in seq.iter().enumerate() {
        let v = b[1 + i];
        if v != *code {
            keymap.insert(code.to_string(), v);
        } else {
            keymap.remove(&code.to_string());
        }
    }
    let angle = u16::from_le_bytes([b[o], b[o + 1]]);
    let f = &b[o + 2..o + 7];
    let mut w = WheelParams { angle: Some(angle), ..Default::default() };
    if f.iter().any(|&x| x != 0) {
        w.ffb = Some(f[0]);
        w.spring = Some(f[1]);
        w.damping = Some(f[2]);
        w.friction = Some(f[3]);
        w.inertia = Some(f[4]);
        w.reverse = Some(b.get(o + 7) == Some(&1));
    }
    Some((mode, keymap, w))
}

const VENDOR_NAMES: [(&str, &str); 2] = [("赛车计划2", "Project CARS 2"), ("尘埃拉力赛2.0", "DiRT Rally 2.0")];

/// Downloads every vendor preset (PC, PS4, Xbox) and stores them as TOML files.
pub async fn refresh_vendor() -> Result<usize, String> {
    let dir = data_dir().join("vendor");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();
    let mut n = 0;
    for (code, platform) in [(20, "pc"), (21, "ps4"), (22, "xbox")] {
        let url = format!("https://api.e-pxn.com.cn/mobile/v1/keyboard/getKeyboard?pageNum=1&pageSize=200&status=1&type=1&driveType={code}");
        let v: serde_json::Value = client
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Could not reach the vendor server: {e}"))?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        for r in v["content"]["list"].as_array().cloned().unwrap_or_default() {
            let (Some(id), Some(raw)) = (r["id"].as_str().map(str::to_string).or_else(|| r["id"].as_u64().map(|x| x.to_string())), r["keboardValue"].as_str()) else { continue };
            let Some((mode, keymap, wheel)) = decode_blob(raw) else { continue };
            let local = r["keyboardName"].as_str().unwrap_or_default().to_string();
            let name = r["englishKeyboardName"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| VENDOR_NAMES.iter().find(|(z, _)| *z == local).map(|(_, e)| e.to_string()))
                .unwrap_or_else(|| local.clone());
            let mut names = BTreeMap::new();
            if local != name && !local.is_empty() {
                names.insert("zh".into(), local.clone());
            }
            let mut notes = BTreeMap::new();
            for (lang, key) in [("en", "englishDescription"), ("zh", "description")] {
                if let Some(s) = r[key].as_str().map(str::trim).filter(|s| !s.is_empty()) {
                    notes.insert(lang.to_string(), s.to_string());
                }
            }
            let tutorial = ["englishDescriptionUrl", "descriptionUrl"].iter().find_map(|k| r[*k].as_str().filter(|s| !s.is_empty()).map(str::to_string));
            let p = Preset {
                format: 1,
                id: format!("vendor-{id}"),
                name,
                platform: platform.into(),
                source: "vendor".into(),
                game: None,
                tutorial,
                names,
                notes,
                links: vec![],
                wheel,
                keymap,
                ffb_server: None,
                vendor: Some(VendorInfo { id: id.clone(), mode, raw: raw.to_string() }),
                origin: String::new(),
            };
            let body = toml::to_string_pretty(&p).map_err(|e| e.to_string())?;
            std::fs::write(dir.join(format!("vendor-{platform}-{}.toml", safe_id(&id))), body).map_err(|e| e.to_string())?;
            n += 1;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_vendor_blobs() {
        // Euro Truck Simulator 2 (PC), mode 3.
        let (m, km, w) = decode_blob("03000102030405060708090A0B0C0D11102F0E0F2F1415161718191A1B1C1D1E1F202122238403641E50505000").unwrap();
        assert_eq!(m, 3);
        assert!(km.is_empty());
        assert_eq!(w, WheelParams { angle: Some(900), ffb: Some(100), spring: Some(30), damping: Some(80), friction: Some(80), inertia: Some(80), reverse: Some(false) });
        // Xbox blob without force values: angle only.
        let (_, _, w) = decode_blob("08000102030405060708090A0B0C0D11102F0E0F2F1415161718191A1B1C1D1E1F2021222384030000000000").unwrap();
        assert_eq!(w, WheelParams { angle: Some(900), ..Default::default() });
        assert!(decode_blob("0300").is_none());
    }

    #[test]
    fn bundled_presets_parse() {
        let l = list();
        assert!(l.errors.is_empty(), "{:?}", l.errors);
        assert!(l.presets.len() >= 20);
        let fs22 = l.presets.iter().find(|p| p.id == "studio-fs22").unwrap();
        assert!(fs22.ffb_server.as_ref().unwrap().feel.is_some());
        assert!(fs22.links.iter().any(|x| x.url.contains("github.com")));
    }
}
