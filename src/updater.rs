use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use base64::Engine as _;
use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub const GITHUB_REPO: &str = "WHCanRC-Media/BookofPraise";
#[cfg(feature = "auto-update")]
pub const GITHUB_PAT: &str = env!("BOP_PAT");
#[cfg(not(feature = "auto-update"))]
pub const GITHUB_PAT: &str = "";
const VERSION_FILE: &str = "lilypond_version.txt";

/// Root directory for update operations (version file + extracted content).
fn update_root() -> PathBuf {
    let dir = crate::render_ly::data_dir();
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Read the locally stored version tag from `lilypond_version.txt`.
fn current_local_version() -> Option<String> {
    let path = update_root().join(VERSION_FILE);
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Persist the given version tag to `lilypond_version.txt` for future update checks.
fn save_local_version(tag: &str) {
    let _ = std::fs::write(update_root().join(VERSION_FILE), tag);
}

#[derive(serde::Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    zipball_url: String,
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

    Ok(Some((release.tag_name, release.zipball_url)))
}

/// Download the source-code zip from a GitHub release and extract the
/// `lilypond/` and `photos/` directories, replacing the local copies.
pub fn download_and_extract(zipball_url: &str, tag: &str) -> Result<(), String> {
    let resp = ureq::get(zipball_url)
        .set("Authorization", &format!("Bearer {GITHUB_PAT}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "bop-rustapp")
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;

    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read failed: {e}"))?;

    // The parent directory that contains lilypond/ and photos/
    let root = update_root();

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| format!("Zip error: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
        let name = file.name().to_string();

        // Strip the variable top-level prefix (e.g. "owner-repo-sha/")
        let after_prefix = match name.find('/') {
            Some(pos) => &name[pos + 1..],
            None => continue,
        };

        // Only extract lilypond/ and photos/ directories
        let rel = if let Some(rest) = after_prefix.strip_prefix("lilypond/") {
            if rest.is_empty() { continue; }
            PathBuf::from("lilypond").join(rest)
        } else if let Some(rest) = after_prefix.strip_prefix("photos/") {
            if rest.is_empty() { continue; }
            PathBuf::from("photos").join(rest)
        } else {
            continue;
        };

        let out_path = root.join(&rel);
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

// ── GitHub API helpers ───────────────────────────────────────────────

fn github_api_get(endpoint: &str) -> Result<serde_json::Value, String> {
    let url = format!("https://api.github.com{endpoint}");
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {GITHUB_PAT}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "bop-rustapp")
        .call()
        .map_err(|e| format!("GET {endpoint}: {e}"))?;
    resp.into_json().map_err(|e| format!("JSON parse: {e}"))
}

fn github_api_post(endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let url = format!("https://api.github.com{endpoint}");
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {GITHUB_PAT}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "bop-rustapp")
        .send_json(body)
        .map_err(|e| format!("POST {endpoint}: {e}"))?;
    resp.into_json().map_err(|e| format!("JSON parse: {e}"))
}

// ── Collect edited files for PR ─────────────────────────────────────

/// Read patchable files (.ly, song.yaml) from a directory into a map of name → content.
fn read_patchable_files(dir: &Path) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return files };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".ly") || name == "song.yaml" {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                files.insert(name, content);
            }
        }
    }
    files
}

/// Check whether any patchable files differ between the edited dirs and their originals.
pub fn has_changes(edited_dirs: &HashSet<PathBuf>, originals_dir: Option<&Path>) -> bool {
    for dir in edited_dirs {
        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
        let orig_files = originals_dir
            .map(|base| read_patchable_files(&base.join(dir_name.as_ref())))
            .unwrap_or_default();
        let current_files = read_patchable_files(dir);

        if orig_files.len() != current_files.len() {
            return true;
        }
        for (name, old) in &orig_files {
            match current_files.get(name) {
                Some(new) if new == old => {}
                _ => return true,
            }
        }
    }
    false
}

/// Collect all current patchable files from edited dirs as repo-relative path → content.
/// E.g. `lilypond/psalm42/notes.ly` → file contents.
pub fn collect_pr_files(edited_dirs: &HashSet<PathBuf>) -> HashMap<String, String> {
    let mut files = HashMap::new();
    for dir in edited_dirs {
        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
        // Find the parent component that is "lilypond" to build repo-relative path
        let repo_prefix = dir
            .ancestors()
            .find(|a| a.file_name().is_some_and(|n| n == "lilypond"))
            .map(|_| format!("lilypond/{dir_name}"))
            .unwrap_or(dir_name.clone());

        for (name, content) in read_patchable_files(dir) {
            files.insert(format!("{repo_prefix}/{name}"), content);
        }
    }
    files
}

/// Generate a timestamp-based branch name like `bop-edit/20260326-143052`.
pub fn generate_branch_name() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let (year, month, day) = crate::model::epoch_days_to_date(days);
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    format!("bop-edit/{year:04}{month:02}{day:02}-{h:02}{m:02}{s:02}")
}

// ── Create PR with files via Git Data API ───────────────────────────

/// Create a GitHub Pull Request with all given files in a single commit.
/// Returns the PR URL on success.
pub fn create_pr_with_files(
    base_branch: &str,
    new_branch: &str,
    files: &HashMap<String, String>,
    title: &str,
    body: &str,
) -> Result<String, String> {
    if files.is_empty() {
        return Err("No files to commit".into());
    }

    let repo = GITHUB_REPO;

    // Step 1: Get base branch HEAD SHA
    let ref_json = github_api_get(&format!("/repos/{repo}/git/ref/heads/{base_branch}"))?;
    let base_sha = ref_json["object"]["sha"]
        .as_str()
        .ok_or("Missing base ref SHA")?
        .to_string();

    // Step 2: Get base commit's tree SHA
    let commit_json = github_api_get(&format!("/repos/{repo}/git/commits/{base_sha}"))?;
    let base_tree_sha = commit_json["tree"]["sha"]
        .as_str()
        .ok_or("Missing base tree SHA")?
        .to_string();

    // Step 3: Create blobs for each file
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut tree_entries = Vec::new();
    for (path, content) in files {
        let blob_json = github_api_post(
            &format!("/repos/{repo}/git/blobs"),
            &serde_json::json!({
                "content": b64.encode(content.as_bytes()),
                "encoding": "base64"
            }),
        )?;
        let blob_sha = blob_json["sha"]
            .as_str()
            .ok_or_else(|| format!("Missing blob SHA for {path}"))?
            .to_string();
        tree_entries.push(serde_json::json!({
            "path": path,
            "mode": "100644",
            "type": "blob",
            "sha": blob_sha
        }));
    }

    // Step 4: Create tree
    let tree_json = github_api_post(
        &format!("/repos/{repo}/git/trees"),
        &serde_json::json!({
            "base_tree": base_tree_sha,
            "tree": tree_entries
        }),
    )?;
    let new_tree_sha = tree_json["sha"]
        .as_str()
        .ok_or("Missing new tree SHA")?
        .to_string();

    // Step 5: Create commit
    let commit_json = github_api_post(
        &format!("/repos/{repo}/git/commits"),
        &serde_json::json!({
            "message": title,
            "tree": new_tree_sha,
            "parents": [base_sha]
        }),
    )?;
    let new_commit_sha = commit_json["sha"]
        .as_str()
        .ok_or("Missing new commit SHA")?
        .to_string();

    // Step 6: Create branch reference
    github_api_post(
        &format!("/repos/{repo}/git/refs"),
        &serde_json::json!({
            "ref": format!("refs/heads/{new_branch}"),
            "sha": new_commit_sha
        }),
    )?;

    // Step 7: Create Pull Request
    let pr_json = github_api_post(
        &format!("/repos/{repo}/pulls"),
        &serde_json::json!({
            "title": title,
            "body": body,
            "head": new_branch,
            "base": base_branch
        }),
    )?;
    let pr_url = pr_json["html_url"]
        .as_str()
        .ok_or("Missing PR URL")?
        .to_string();

    Ok(pr_url)
}

// ── Hymn usage reporting ────────────────────────────────────────────

const SMTP_HOST: &str = "smtp.purelymail.com";
const SMTP_USER: &str = "bopnotifications@microridge.ca";
const SMTP_PASS: &str = "s^]Xd;?@_5UW;MW";

/// Return the path to the hymn usage file.
pub fn hymn_usage_path() -> PathBuf {
    std::env::var("HYMN_USAGE_TXT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| crate::render_ly::data_dir().join("HymnUsage.txt"))
}

/// Check whether the usage file has data and we haven't already sent
/// a report for the current year.
pub fn should_report_hymn_usage() -> bool {
    let path = hymn_usage_path();
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) if !c.trim().is_empty() => c,
        _ => return false,
    };
    let current_year = current_year_str();
    // Only prompt if file has entries from a previous year
    let has_old_entries = contents.lines().any(|line| !line.contains(&current_year));
    if !has_old_entries {
        return false;
    }
    // Check if we already sent for this year
    let sent_year = std::fs::read_to_string(usage_sent_marker()).unwrap_or_default();
    sent_year.trim() != current_year
}

fn usage_sent_marker() -> PathBuf {
    crate::render_ly::cache_dir().join("hymn_usage_sent")
}

fn current_year_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let (year, _, _) = crate::model::epoch_days_to_date(days);
    year.to_string()
}

/// Send the hymn usage report to the given email address, then clear the file.
pub fn email_hymn_usage(to_address: &str) -> Result<(), String> {
    let path = hymn_usage_path();
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read usage file: {e}"))?;
    if contents.trim().is_empty() {
        return Ok(());
    }

    let attachment = Attachment::new("HymnUsage.txt".to_string())
        .body(contents, ContentType::TEXT_PLAIN);

    let email = Message::builder()
        .from(SMTP_USER.parse().map_err(|e| format!("From address error: {e}"))?)
        .to(to_address.parse().map_err(|e| format!("To address error: {e}"))?)
        .subject(format!("Hymn Usage Report – {}", current_year_str()))
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::plain(
                    "Attached is the hymn usage report from the Book of Praise application."
                        .to_string(),
                ))
                .singlepart(attachment),
        )
        .map_err(|e| format!("Email build error: {e}"))?;

    let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASS.to_string());
    let mailer = SmtpTransport::starttls_relay(SMTP_HOST)
        .map_err(|e| format!("SMTP relay error: {e}"))?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| format!("Send error: {e}"))?;
    // Record that we sent for this year
    let _ = std::fs::write(usage_sent_marker(), current_year_str());
    Ok(())
}
