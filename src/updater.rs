use std::io::Read as _;

use crate::model::base_dir;

pub const GITHUB_REPO: &str = "vanjoe/bookOfPraise";
pub const GITHUB_PAT: &str = "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";
const VERSION_FILE: &str = "lilypond_version.txt";

fn current_local_version() -> Option<String> {
    let path = base_dir(true).parent()?.join(VERSION_FILE);
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn save_local_version(tag: &str) {
    if let Some(parent) = base_dir(true).parent() {
        let _ = std::fs::write(parent.join(VERSION_FILE), tag);
    }
}

#[derive(serde::Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(serde::Deserialize)]
struct ReleaseAsset {
    name: String,
    url: String,
}

/// Check GitHub for a newer release. Returns (tag, asset_download_url) if available.
pub fn check_for_update() -> Result<Option<(String, String)>, String> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {GITHUB_PAT}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "bop-rustapp")
        .call()
        .map_err(|e| format!("API request failed: {e}"))?;

    let release: ReleaseInfo = resp.into_json().map_err(|e| format!("JSON parse error: {e}"))?;

    let local = current_local_version();
    if local.as_deref() == Some(&release.tag_name) {
        return Ok(None);
    }

    // Find the lilypond zip asset
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.starts_with("lilypond") && a.name.ends_with(".zip"))
        .ok_or("No lilypond zip asset found in release")?;

    Ok(Some((release.tag_name, asset.url.clone())))
}

/// Download and extract the lilypond zip, replacing the local lilypond/ directory.
pub fn download_and_extract(asset_url: &str, tag: &str) -> Result<(), String> {
    // Download the asset (need Accept: application/octet-stream for the redirect)
    let resp = ureq::get(asset_url)
        .set("Authorization", &format!("Bearer {GITHUB_PAT}"))
        .set("Accept", "application/octet-stream")
        .set("User-Agent", "bop-rustapp")
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;

    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read failed: {e}"))?;

    let target_dir = base_dir(true);

    // Extract zip over the existing directory
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| format!("Zip error: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
        let name = file.name().to_string();

        // Strip the top-level directory from the zip (e.g. "lilypond/psalm1/..." → "psalm1/...")
        let rel = name
            .strip_prefix("lilypond/")
            .unwrap_or(&name);
        if rel.is_empty() {
            continue;
        }

        let out_path = target_dir.join(rel);
        if file.is_dir() {
            let _ = std::fs::create_dir_all(&out_path);
        } else {
            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| format!("Extract read error: {e}"))?;
            std::fs::write(&out_path, buf).map_err(|e| format!("Write error: {e}"))?;
        }
    }

    save_local_version(tag);
    Ok(())
}
