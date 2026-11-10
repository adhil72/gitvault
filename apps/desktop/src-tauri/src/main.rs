// Prevents an additional console window on Windows in release. DO NOT REMOVE.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = gitvault_desktop_lib::run() {
        eprintln!("failed to start GitVault: {error}");
        std::process::exit(1);
    }
}
