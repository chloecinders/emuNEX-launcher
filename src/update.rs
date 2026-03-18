use serde::Deserialize;
use std::{
    fs,
    io::{self, Cursor},
    path::Path,
};

#[cfg(not(target_os = "windows"))]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use crate::config::LauncherConfig;

#[derive(Debug, Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    status: String,
    conclusion: Option<String>,
    artifacts_url: String,
}

#[derive(Debug, Deserialize)]
struct ArtifactsResponse {
    artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    name: String,
    archive_download_url: String,
}

pub fn pull_update() -> Result<(), String> {
    let cfg = LauncherConfig::load().map_err(|e| format!("config error: {e}"))?;

    let repo = cfg.repository.ok_or("No repository configured in Config.toml")?;
    let client = reqwest::blocking::Client::new();

    // --- 1. Find the latest successful workflow run ---
    let mut req = client
        .get(format!(
            "https://api.github.com/repos/{repo}/actions/runs?per_page=1"
        ))
        .header("User-Agent", "emunex-launcher");

    if let Some(token) = &cfg.github_token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let runs: WorkflowRunsResponse = req
        .send()
        .map_err(|e| format!("runs fetch error: {e}"))?
        .json()
        .map_err(|e| format!("runs parse error: {e}"))?;

    let run = runs
        .workflow_runs
        .first()
        .ok_or("No workflow runs found")?;

    if run.status != "completed" || run.conclusion.as_deref() != Some("success") {
        return Err(format!(
            "Latest CI run is not successful (status={}, conclusion={:?})",
            run.status, run.conclusion
        ));
    }

    // --- 2. Find the right artifact ---
    let mut art_req = client
        .get(&run.artifacts_url)
        .header("User-Agent", "emunex-launcher");

    if let Some(token) = &cfg.github_token {
        art_req = art_req.header("Authorization", format!("Bearer {token}"));
    }

    let art_json: ArtifactsResponse = art_req
        .send()
        .map_err(|e| format!("artifacts fetch error: {e}"))?
        .json()
        .map_err(|e| format!("artifacts parse error: {e}"))?;

    // Pick the artifact that doesn't end in .exe (the linux build)
    let artifact = art_json
        .artifacts
        .iter()
        .find(|a| !a.name.ends_with(".exe"))
        .ok_or("No valid artifact found in the run")?;

    // --- 3. Download the zip ---
    let mut dl_req = client
        .get(&artifact.archive_download_url)
        .header("User-Agent", "emunex-launcher");

    if let Some(token) = &cfg.github_token {
        dl_req = dl_req.header("Authorization", format!("Bearer {token}"));
    }

    let bytes = dl_req
        .send()
        .map_err(|e| format!("download error: {e}"))?
        .bytes()
        .map_err(|e| format!("body read error: {e}"))?;

    // --- 4. Extract the zip ---
    let base_dir = launcher_dir();
    let temp_bin = base_dir.join("emunex-server.new");
    let mut binary_found = false;

    let reader = Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("zip error: {e}"))?;

    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| format!("zip entry error: {e}"))?;
        let name = file.name().to_string();

        if name == "target/release/emunex-server" || name == "target/release/emunex-server.exe" {
            let mut outfile =
                fs::File::create(&temp_bin).map_err(|e| format!("create temp binary: {e}"))?;
            io::copy(&mut file, &mut outfile).map_err(|e| format!("write temp binary: {e}"))?;
            binary_found = true;
        } else if name.starts_with("templates") || name.starts_with("public") {
            let target_path = base_dir.join(&name);

            if name.ends_with('/') {
                fs::create_dir_all(&target_path)
                    .map_err(|e| format!("create dir {name}: {e}"))?;
            } else {
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("create parent for {name}: {e}"))?;
                    }
                }
                let mut outfile = fs::File::create(&target_path)
                    .map_err(|e| format!("create file {name}: {e}"))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("write file {name}: {e}"))?;
            }
        }
    }

    if !binary_found {
        return Err("Binary not found in artifact zip".into());
    }

    // --- 5. Swap the binary ---
    let server_bin = if cfg!(target_os = "windows") {
        base_dir.join("emunex-server.exe")
    } else {
        base_dir.join("emunex-server")
    };

    #[cfg(not(target_os = "windows"))]
    fs::set_permissions(&temp_bin, Permissions::from_mode(0o755))
        .map_err(|e| format!("chmod error: {e}"))?;

    fs::rename(&temp_bin, &server_bin).map_err(|e| format!("rename error: {e}"))?;

    println!("[launcher] binary swapped successfully");
    Ok(())
}

fn launcher_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}
