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

    tauri_build::build()
}
