// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--import" {
        remember_lib::import_cli(&args[2]);
        return;
    }
    remember_lib::run()
}
