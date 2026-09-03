use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::Manager;

pub const MODEL_PACK_ID: &str = "en-id-opus-v1";
const MODEL_PACK_URL: Option<&str> = option_env!("CLIPLINGO_MODEL_PACK_URL");
const MODEL_PACK_SHA256: Option<&str> = option_env!("CLIPLINGO_MODEL_PACK_SHA256");

const REQUIRED_FILES: &[&str] = &[
    "manifest.json",
    "stages/en-id/config.json",
    "stages/en-id/model.bin",
    "stages/en-id/source.spm",
    "stages/en-id/target.spm",
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPackStatus {
    pub id: &'static str,
    pub installed: bool,
    pub install_supported: bool,
}

pub fn model_pack_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("unable to resolve ClipLingo data directory: {error}"))?;
    Ok(data_dir.join("models").join(MODEL_PACK_ID))
}

pub fn status(app: &tauri::AppHandle) -> Result<ModelPackStatus, String> {
    let directory = model_pack_directory(app)?;
    Ok(status_for_directory(&directory))
}

pub async fn install(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    let directory = model_pack_directory(&app)?;
    let url = MODEL_PACK_URL.ok_or_else(|| {
        "this ClipLingo build does not include a model-pack download source".to_string()
    })?;
    let expected_sha256 = MODEL_PACK_SHA256.ok_or_else(|| {
        "this ClipLingo build does not include model-pack integrity metadata".to_string()
    })?;

    let directory_for_install = directory.clone();
    tauri::async_runtime::spawn_blocking(move || {
        install_verified_archive(&directory_for_install, url, expected_sha256)
    })
    .await
    .map_err(|error| format!("model installation task failed: {error}"))??;

    Ok(status_for_directory(&directory))
}

pub async fn remove(app: tauri::AppHandle) -> Result<ModelPackStatus, String> {
    let directory = model_pack_directory(&app)?;
    let directory_for_remove = directory.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if directory_for_remove.exists() {
            fs::remove_dir_all(&directory_for_remove)
                .map_err(|error| format!("unable to remove model pack: {error}"))?;
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|error| format!("model removal task failed: {error}"))??;

    Ok(status_for_directory(&directory))
}

pub fn is_complete(directory: &Path) -> bool {
    REQUIRED_FILES
        .iter()
        .all(|relative| directory.join(relative).is_file())
}

fn status_for_directory(directory: &Path) -> ModelPackStatus {
    ModelPackStatus {
        id: MODEL_PACK_ID,
        installed: is_complete(directory),
        install_supported: MODEL_PACK_URL.is_some() && MODEL_PACK_SHA256.is_some(),
    }
}

fn install_verified_archive(
    destination: &Path,
    url: &str,
    expected_sha256: &str,
) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| "model pack destination has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("unable to create model directory: {error}"))?;

    let staging = parent.join(format!("{MODEL_PACK_ID}.staging"));
    let archive = parent.join(format!("{MODEL_PACK_ID}.download.zip"));
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .map_err(|error| format!("unable to clear model staging directory: {error}"))?;
    }
    if archive.exists() {
        fs::remove_file(&archive)
            .map_err(|error| format!("unable to clear model download archive: {error}"))?;
    }

    let script = r#"
$ErrorActionPreference = 'Stop'
$url = $env:CLIPLINGO_MODEL_URL
$expected = $env:CLIPLINGO_MODEL_SHA256.ToLowerInvariant()
$archive = $env:CLIPLINGO_MODEL_ARCHIVE
$staging = $env:CLIPLINGO_MODEL_STAGING
Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archive
$actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "model pack SHA256 mismatch" }
Expand-Archive -LiteralPath $archive -DestinationPath $staging -Force
"#;

    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("CLIPLINGO_MODEL_URL", url)
        .env("CLIPLINGO_MODEL_SHA256", expected_sha256)
        .env("CLIPLINGO_MODEL_ARCHIVE", &archive)
        .env("CLIPLINGO_MODEL_STAGING", &staging)
        .status()
        .map_err(|error| format!("unable to start model downloader: {error}"))?;

    let _ = fs::remove_file(&archive);
    if !status.success() {
        let _ = fs::remove_dir_all(&staging);
        return Err("model download, integrity verification, or extraction failed".to_string());
    }
    if !is_complete(&staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err("downloaded model pack is incomplete".to_string());
    }

    if destination.exists() {
        fs::remove_dir_all(destination)
            .map_err(|error| format!("unable to replace existing model pack: {error}"))?;
    }
    fs::rename(&staging, destination)
        .map_err(|error| format!("unable to activate model pack: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn incomplete_pack_is_not_ready() {
        let root = temporary_directory("incomplete");
        fs::create_dir_all(&root).unwrap();
        assert!(!is_complete(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn required_files_define_a_complete_pack() {
        let root = temporary_directory("complete");
        for relative in REQUIRED_FILES {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"fixture").unwrap();
        }
        assert!(is_complete(&root));
        fs::remove_dir_all(root).unwrap();
    }

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("cliplingo-{label}-{nonce}"))
    }
}
