mod config;
mod install;
mod update;

use std::{
    process::{Command, ExitStatus},
    thread,
    time::Duration,
};

const UPDATE_EXIT_CODE: i32 = 2;

fn main() {
    println!("[launcher] emunex-launcher starting");

    if let Err(e) = install::ensure_ready() {
        eprintln!("[launcher] install check failed: {e}");
        std::process::exit(1);
    }

    loop {
        println!("[launcher] spawning emunex-server...");

        let status = run_server();

        match status {
            Ok(s) => {
                let code = s.code().unwrap_or(-1);
                println!("[launcher] server exited with code {code}");

                if code == UPDATE_EXIT_CODE {
                    println!("[launcher] update signal received, pulling latest build...");
                    match update::pull_update() {
                        Ok(()) => println!("[launcher] update successful, restarting server"),
                        Err(e) => eprintln!("[launcher] update failed: {e}"),
                    }
                } else {
                    println!("[launcher] server exited normally, restarting in 1s...");
                    thread::sleep(Duration::from_secs(1));
                }
            }
            Err(e) => {
                eprintln!("[launcher] failed to start server: {e}");
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
}

fn run_server() -> std::io::Result<ExitStatus> {
    let bin = server_binary_path();
    Command::new(&bin).status()
}

pub fn server_binary_path() -> std::path::PathBuf {
    let name = if cfg!(target_os = "windows") {
        "emunex-server.exe"
    } else {
        "emunex-server"
    };

    let mut path = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    path.push(name);
    path
}
