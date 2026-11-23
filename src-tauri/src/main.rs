#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(profile_code) = adabarprint_tauri_lib::parse_cli_profile_arg(&args) {
        let code = adabarprint_tauri_lib::run_profile_cli(&profile_code);
        std::process::exit(code);
    }
    adabarprint_tauri_lib::run()
}
