use crate::update;
use std::path::Path;

pub fn ensure_ready() -> Result<(), String> {
    let base = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let server_bin = if cfg!(target_os = "windows") {
        base.join("emunex-server.exe")
    } else {
        base.join("emunex-server")
    };

    let config_path = base.join("Config.toml");
    if !config_path.exists() {
        let default_config = base.join("Config.default.toml");
        if default_config.exists() {
            let _ = std::fs::copy(&default_config, &config_path);
        }
    }

    let templates_dir = base.join("templates");
    if !server_bin.exists() || !templates_dir.exists() {
        println!("[launcher] server binary or resources missing, performing installation...");
        update::pull_update()?;
    }

    println!(
        "[launcher] pre-flight OK - server binary at {}",
        server_bin.display()
    );
    Ok(())
}
