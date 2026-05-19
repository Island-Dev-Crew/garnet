// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--studio-smoke") {
        match garnet_studio_lib::run_smoke() {
            Ok(path) => {
                println!("Garnet Studio smoke passed");
                println!("evidence={path}");
                return;
            }
            Err(err) => {
                eprintln!("Garnet Studio smoke failed: {err}");
                std::process::exit(1);
            }
        }
    }

    garnet_studio_lib::run()
}
