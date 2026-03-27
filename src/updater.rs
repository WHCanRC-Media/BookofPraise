use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::model::base_dir;

pub const GITHUB_REPO: &str = "vanjoe/bookOfPraise";
pub const GITHUB_PAT: &str = "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";
const VERSION_FILE: &str = "lilypond_version.txt";

/// Read the locally stored version tag from `lilypond_version.txt`.
fn current_local_version() -> Option<String> {
    let path = base_dir(true).parent()?.join(VERSION_FILE);
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Persist the given version tag to `lilypond_version.txt` for future update checks.
fn save_local_version(tag: &str) {
    if let Some(parent) = base_dir(true).parent() {
        let _ = std::fs::write(parent.join(VERSION_FILE), tag);
    }
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
    let root = base_dir(true)
        .parent()
        .ok_or("Cannot determine root directory")?
        .to_path_buf();

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

// ── Email edits as patch ────────────────────────────────────────────

const SMTP_HOST: &str = "smtp.purelymail.com";
const SMTP_USER: &str = "bopnotifications@microridge.ca";
const SMTP_PASS: &str = "s^]Xd;?@_5UW;MW";
const NOTIFY_TO: &str = "joelvandergriendt@microridge.ca";

/// Read patchable files (.ly, song.yaml) from a directory into a map of name → content.
fn read_patchable_files(dir: &Path) -> std::collections::HashMap<String, String> {
    let mut files = std::collections::HashMap::new();
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

/// Produce a unified diff for a single file using the `similar` crate.
fn diff_file(rel_path: &str, old: &str, new: &str) -> String {
    let old_path = if old.is_empty() { "/dev/null".to_string() } else { format!("a/{rel_path}") };
    let new_path = if new.is_empty() { "/dev/null".to_string() } else { format!("b/{rel_path}") };
    similar::TextDiff::from_lines(old, new)
        .unified_diff()
        .header(&old_path, &new_path)
        .context_radius(3)
        .to_string()
}

/// Build a unified diff patch comparing current files against originals in the
/// temp snapshot directory.
pub fn build_patch(edited_dirs: &HashSet<PathBuf>, originals_dir: Option<&Path>) -> String {
    let mut patch = String::new();
    for dir in edited_dirs {
        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
        let orig_files = originals_dir
            .map(|base| read_patchable_files(&base.join(dir_name.as_ref())))
            .unwrap_or_default();
        let current_files = read_patchable_files(dir);

        let mut all_names: Vec<_> = orig_files.keys()
            .chain(current_files.keys())
            .cloned()
            .collect();
        all_names.sort();
        all_names.dedup();

        for name in all_names {
            let old = orig_files.get(&name).map(|s| s.as_str()).unwrap_or("");
            let new = current_files.get(&name).map(|s| s.as_str()).unwrap_or("");
            if old == new {
                continue;
            }
            patch.push_str(&diff_file(&format!("{dir_name}/{name}"), old, new));
        }
    }
    patch
}

/// Send an email with a unified diff patch of edited files.
pub fn email_edits(
    edited_dirs: &HashSet<PathBuf>,
    originals_dir: Option<&Path>,
) -> Result<(), String> {
    let patch = build_patch(edited_dirs, originals_dir);
    if patch.is_empty() {
        return Ok(());
    }

    let dir_names: Vec<String> = edited_dirs
        .iter()
        .filter_map(|d| d.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    let subject = format!("BOP edits: {}", dir_names.join(", "));

    let attachment = Attachment::new("edits.patch".to_string())
        .body(patch, ContentType::TEXT_PLAIN);

    let email = Message::builder()
        .from(SMTP_USER.parse().map_err(|e| format!("From address error: {e}"))?)
        .to(NOTIFY_TO.parse().map_err(|e| format!("To address error: {e}"))?)
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(SinglePart::plain("Lilypond edits from Book of Praise app.".to_string()))
                .singlepart(attachment),
        )
        .map_err(|e| format!("Email build error: {e}"))?;

    let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASS.to_string());
    let mailer = SmtpTransport::starttls_relay(SMTP_HOST)
        .map_err(|e| format!("SMTP relay error: {e}"))?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| format!("Send error: {e}"))?;
    Ok(())
}

// ── Hymn usage reporting ────────────────────────────────────────────

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
