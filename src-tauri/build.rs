use std::fmt::Write;

fn main() {
    // Compile every presets/*.toml into the binary (see src/presets.rs).
    let dir = std::path::Path::new("presets");
    println!("cargo:rerun-if-changed=presets");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "toml")).collect())
        .unwrap_or_default();
    files.sort();
    let mut code = String::from("pub const BUNDLED: &[(&str, &str)] = &[\n");
    for f in &files {
        let abs = std::fs::canonicalize(f).unwrap();
        let name = f.file_name().unwrap().to_string_lossy();
        writeln!(code, "    ({name:?}, include_str!({:?})),", abs.display().to_string()).unwrap();
    }
    code.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bundled_presets.rs");
    std::fs::write(out, code).unwrap();

    // The tablet dashboard: its built-in game profiles, its web page, and the ETS2/ATS plugin
    // (src/dash/). The plugin is optional at build time (plugins/ets2-dash builds it).
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    embed("dash-profiles", "DASH_PROFILES", "include_str", &out_dir.join("dash_profiles.rs"));
    embed("dash-web", "DASH_WEB", "include_bytes", &out_dir.join("dash_web.rs"));
    embed("resources/ets2", "ETS2_PLUGIN", "include_bytes", &out_dir.join("ets2_plugin.rs"));

    tauri_build::build()
}

/// Writes `pub const NAME: &[(&str, &[u8] | &str)]` with every file in `dir` (flat).
fn embed(dir: &str, name: &str, how: &str, out: &std::path::Path) {
    println!("cargo:rerun-if-changed={dir}");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    files.sort();
    let ty = if how == "include_str" { "&str" } else { "&[u8]" };
    let mut code = format!("pub const {name}: &[(&str, {ty})] = &[\n");
    for f in &files {
        let abs = std::fs::canonicalize(f).unwrap();
        let file = f.file_name().unwrap().to_string_lossy();
        writeln!(code, "    ({file:?}, {how}!({:?})),", abs.display().to_string()).unwrap();
    }
    code.push_str("];\n");
    std::fs::write(out, code).unwrap();
}
