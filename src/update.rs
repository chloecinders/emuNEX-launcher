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
    let repo = cfg.repository.ok_or("No repository configured")?;
    let client = reqwest::blocking::Client::new();

    let mut req = client
        .get(format!(
            "https://api.github.com/repos/{repo}/actions/runs?per_page=1"
        ))
        .header("User-Agent", "emunex-launcher");
    if let Some(token) = &cfg.github_token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req.send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!(
            "GitHub API error (runs): {} - {}",
            resp.status(),
            resp.text().unwrap_or_default()
        ));
    }
    let runs: WorkflowRunsResponse = resp
        .json()
        .map_err(|e| format!("JSON decode error (runs): {e}"))?;
    let run = runs.workflow_runs.first().ok_or("No runs")?;

    let mut art_req = client
        .get(&run.artifacts_url)
        .header("User-Agent", "emunex-launcher");
    if let Some(token) = &cfg.github_token {
        art_req = art_req.header("Authorization", format!("Bearer {token}"));
    }

    let art_resp = art_req.send().map_err(|e| e.to_string())?;
    if !art_resp.status().is_success() {
        return Err(format!(
            "GitHub API error (artifacts): {} - {}",
            art_resp.status(),
            art_resp.text().unwrap_or_default()
        ));
    }
    let art_json: ArtifactsResponse = art_resp
        .json()
        .map_err(|e| format!("JSON decode error (artifacts): {e}"))?;
    let artifact = art_json
        .artifacts
        .iter()
        .find(|a| !a.name.ends_with(".exe"))
        .ok_or("No artifact")?;

    let mut dl_req = client
        .get(&artifact.archive_download_url)
        .header("User-Agent", "emunex-launcher");
    if let Some(token) = &cfg.github_token {
        dl_req = dl_req.header("Authorization", format!("Bearer {token}"));
    }

    let bytes = dl_req
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    let base_dir = launcher_dir();

    let pub_path = base_dir.join("public");
    let tmpl_path = base_dir.join("templates");
    if pub_path.exists() {
        fs::remove_dir_all(&pub_path).ok();
    }
    if tmpl_path.exists() {
        fs::remove_dir_all(&tmpl_path).ok();
    }

    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut binary_found = false;
    let temp_bin = base_dir.join("emunex-server.new");

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name();

        if name.ends_with("emunex-server") || name.ends_with("emunex-server.exe") {
            let mut outfile = fs::File::create(&temp_bin).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            binary_found = true;
        } else {
            let relative_path = if let Some(idx) = name.find("templates/") {
                Some(&name[idx..])
            } else if let Some(idx) = name.find("public/") {
                Some(&name[idx..])
            } else {
                None
            };

            if let Some(rel) = relative_path {
                let target_path = base_dir.join(rel);
                if name.ends_with('/') {
                    fs::create_dir_all(&target_path).ok();
                } else {
                    if let Some(p) = target_path.parent() {
                        fs::create_dir_all(p).ok();
                    }
                    let mut outfile = fs::File::create(&target_path).map_err(|e| e.to_string())?;
                    io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    if !binary_found {
        return Err("No binary".into());
    }

    let server_bin = if cfg!(target_os = "windows") {
        base_dir.join("emunex-server.exe")
    } else {
        base_dir.join("emunex-server")
    };
    #[cfg(not(target_os = "windows"))]
    fs::set_permissions(&temp_bin, fs::Permissions::from_mode(0o755)).ok();

    fs::rename(&temp_bin, &server_bin).map_err(|e| e.to_string())?;
    Ok(())
}

fn launcher_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}
