use std::path::Path;

/// Run pre-flight checks before starting the supervision loop.
/// Returns Ok(()) if everything looks good, or Err with a human-readable message.
pub fn ensure_ready() -> Result<(), String> {
    let base = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    // Check the server binary exists
    let server_bin = if cfg!(target_os = "windows") {
        base.join("emunex-server.exe")
    } else {
        base.join("emunex-server")
    };

    if !server_bin.exists() {
        return Err(format!(
            "emunex-server binary not found at {}\n\
             Place emunex-launcher alongside emunex-server in the same directory.",
            server_bin.display()
        ));
    }

    // Warn (don't fail) if templates/ is missing — server will error on its own
    let templates_dir = base.join("templates");
    if !templates_dir.exists() {
        eprintln!(
            "[launcher] warning: templates/ not found at {}. \
             The server may fail to start.",
            templates_dir.display()
        );
    }

    // Warn if Config.toml is missing
    let config = base.join("Config.toml");
    if !config.exists() {
        // Try to copy Config.default.toml as a starting point
        let default_config = base.join("Config.default.toml");
        if default_config.exists() {
            match std::fs::copy(&default_config, &config) {
                Ok(_) => println!(
                    "[launcher] copied Config.default.toml → Config.toml (please review it)"
                ),
                Err(e) => eprintln!("[launcher] warning: could not copy default config: {e}"),
            }
        } else {
            eprintln!(
                "[launcher] warning: Config.toml not found at {}. \
                 The server may fail to start.",
                config.display()
            );
        }
    }

    println!(
        "[launcher] pre-flight OK — server binary at {}",
        server_bin.display()
    );
    Ok(())
}
