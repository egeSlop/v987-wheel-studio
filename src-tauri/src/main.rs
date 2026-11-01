// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--ffb-server") {
        let rest: Vec<&str> = args[1..].iter().map(String::as_str).collect();
        if let Err(e) = v987_wheel_studio_lib::headless(&rest) {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return;
    }
    v987_wheel_studio_lib::run()
}
