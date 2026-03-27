//! On-demand LilyPond SVG rendering.
//!
//! Combines `notes.ly` + `lyrics_N.ly` + `song.yaml` into a single
//! `.ly` file and invokes `lilypond` to produce an SVG. Skips rendering
//! when the output SVG is already newer than all source files.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Minimum number of `\break`s before lines are combined in pairs.
const COMBINE_LINES_THRESHOLD: usize = 7;

/// How a song's lines should be split across slides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitStyle {
    /// Automatically decide based on line count thresholds.
    #[serde(rename = "default")]
    Default,
    /// Force everything onto a single slide.
    #[serde(rename = "single slide")]
    SingleSlide,
    /// Force rendering across multiple slides.
    #[serde(rename = "multi slide")]
    MultiSlide,
    /// Force combining line pairs into single lines.
    #[serde(rename = "combine lines")]
    CombineLines,
}

impl Default for SplitStyle {
    fn default() -> Self {
        SplitStyle::Default
    }
}

impl SplitStyle {
    pub const ALL: &[SplitStyle] = &[
        SplitStyle::Default,
        SplitStyle::SingleSlide,
        SplitStyle::MultiSlide,
        SplitStyle::CombineLines,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            SplitStyle::Default => "default",
            SplitStyle::SingleSlide => "single slide",
            SplitStyle::MultiSlide => "multi slide",
            SplitStyle::CombineLines => "combine lines",
        }
    }

}

/// Song metadata stored in `song.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composer: Option<String>,
    #[serde(default)]
    pub split_style: SplitStyle,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub verified: std::collections::HashMap<u32, u32>,
}

impl Default for SongMeta {
    fn default() -> Self {
        SongMeta {
            composer: None,
            split_style: SplitStyle::Default,
            verified: std::collections::HashMap::new(),
        }
    }
}

/// Read song metadata from `song.yaml` in the given directory.
/// Falls back to reading legacy `composer.txt` if `song.yaml` doesn't exist.
pub fn read_song_meta(song_dir: &Path) -> SongMeta {
    let yaml_path = song_dir.join("song.yaml");
    if let Ok(content) = fs::read_to_string(&yaml_path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        // Legacy fallback: read composer.txt
        let composer_path = song_dir.join("composer.txt");
        let composer = fs::read_to_string(&composer_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        SongMeta {
            composer,
            split_style: SplitStyle::Default,
            verified: std::collections::HashMap::new(),
        }
    }
}

/// Write song metadata to `song.yaml` in the given directory.
pub fn write_song_meta(song_dir: &Path, meta: &SongMeta) {
    let yaml_path = song_dir.join("song.yaml");
    if let Ok(content) = serde_yaml::to_string(meta) {
        let _ = fs::write(&yaml_path, content);
    }
}

/// Return the platform cache directory for rendered SVGs.
pub fn svg_cache_dir() -> PathBuf {
    cache_dir().join("svg")
}
/// Return the platform data directory for persistent app data.
/// Linux: `$XDG_DATA_HOME/bop` (default `~/.local/share/bop`)
/// Windows: `%APPDATA%\bop`
pub fn data_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Temp"))
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".local").join("share")
            })
    };
    base.join("bop")
}

pub fn cache_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Temp"))
    } else {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".cache")
            })
    };
    base.join("bop")
}

/// Compute a hex-encoded hash of the given string for use as a cache key.
fn content_hash(data: &str) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Return the cached SVG path for the given combined .ly content.
pub fn cached_svg_path(combined_ly_content: &str) -> PathBuf {
    let hash = content_hash(combined_ly_content);
    svg_cache_dir().join(format!("{hash}.svg"))
}

const LILYPOND_VERSION: &str = "2.24.4";

/// Return the cache directory for the LilyPond installation.
fn lilypond_cache_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Temp"))
    } else {
        std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".cache")
            })
    };
    base.join("bop").join("lilypond-bin")
}

/// Check whether LilyPond is available (bundled, cached, or on PATH).
pub fn lilypond_available() -> bool {
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let bin_name = format!("lilypond{exe_suffix}");

    // Bundled next to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("lilypond-bin").join("bin").join(&bin_name).exists() {
                return true;
            }
        }
    }

    // In cache
    if lilypond_cache_dir().join("bin").join(&bin_name).exists() {
        return true;
    }

    // On PATH
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    Command::new(which_cmd).arg("lilypond").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}

/// Download and extract LilyPond into the cache directory.
pub fn download_lilypond() -> Result<(), String> {
    let dest = lilypond_cache_dir();
    let parent = dest.parent().unwrap();
    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create cache dir: {e}"))?;

    eprintln!("Downloading LilyPond {LILYPOND_VERSION}...");

    if cfg!(windows) {
        let url = format!(
            "https://gitlab.com/lilypond/lilypond/-/releases/v{LILYPOND_VERSION}/downloads/lilypond-{LILYPOND_VERSION}-mingw-x86_64.zip"
        );
        let resp = ureq::get(&url).call()
            .map_err(|e| format!("Failed to download LilyPond: {e}"))?;
        let zip_path = dest.with_extension("zip");
        let mut file = fs::File::create(&zip_path)
            .map_err(|e| format!("Failed to create zip file: {e}"))?;
        std::io::copy(&mut resp.into_reader(), &mut file)
            .map_err(|e| format!("Failed to write zip file: {e}"))?;

        let file = fs::File::open(&zip_path)
            .map_err(|e| format!("Failed to open zip file: {e}"))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {e}"))?;

        let extract_dir = dest.with_extension("extract");
        archive.extract(&extract_dir)
            .map_err(|e| format!("Failed to extract zip: {e}"))?;

        let extracted = extract_dir.join(format!("lilypond-{LILYPOND_VERSION}"));
        if extracted.exists() {
            fs::rename(&extracted, &dest)
                .map_err(|e| format!("Failed to rename extracted dir: {e}"))?;
        }
        let _ = fs::remove_dir_all(&extract_dir);
        let _ = fs::remove_file(&zip_path);
    } else {
        let url = format!(
            "https://gitlab.com/lilypond/lilypond/-/releases/v{LILYPOND_VERSION}/downloads/lilypond-{LILYPOND_VERSION}-linux-x86_64.tar.gz"
        );
        let resp = ureq::get(&url).call()
            .map_err(|e| format!("Failed to download LilyPond: {e}"))?;
        let tar_path = dest.with_extension("tar.gz");
        let mut file = fs::File::create(&tar_path)
            .map_err(|e| format!("Failed to create tar file: {e}"))?;
        std::io::copy(&mut resp.into_reader(), &mut file)
            .map_err(|e| format!("Failed to write tar file: {e}"))?;

        let status = Command::new("tar")
            .args(["xzf"])
            .arg(&tar_path)
            .arg("-C")
            .arg(parent)
            .status()
            .map_err(|e| format!("Failed to run tar: {e}"))?;
        if !status.success() {
            return Err("tar extraction failed".into());
        }

        let extracted = parent.join(format!("lilypond-{LILYPOND_VERSION}"));
        if extracted.exists() {
            fs::rename(&extracted, &dest)
                .map_err(|e| format!("Failed to rename extracted dir: {e}"))?;
        }
        let _ = fs::remove_file(&tar_path);
    }

    eprintln!("LilyPond {LILYPOND_VERSION} installed to cache.");
    Ok(())
}

/// Find the lilypond binary: check next to our executable, then cache, then PATH.
fn lilypond_bin() -> PathBuf {
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let bin_name = format!("lilypond{exe_suffix}");

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("lilypond-bin").join("bin").join(&bin_name);
            if bundled.exists() {
                return bundled;
            }
        }
    }

    let cached = lilypond_cache_dir().join("bin").join(&bin_name);
    if cached.exists() {
        return cached;
    }

    PathBuf::from("lilypond")
}

/// Map a LilyPond note name to a pitch class (0–6, matching C=0 .. B=6).
fn note_name_to_pitch_class(name: &str) -> Option<i32> {
    let base = match name.chars().next()? {
        'c' => 0,
        'd' => 1,
        'e' => 2,
        'f' => 3,
        'g' => 4,
        'a' => 5,
        'b' => 6,
        _ => return None,
    };
    Some(base)
}

/// Convert a pitch class (0–6) back to a LilyPond note name.
fn pitch_class_to_name(pc: i32) -> &'static str {
    match pc.rem_euclid(7) {
        0 => "c",
        1 => "d",
        2 => "e",
        3 => "f",
        4 => "g",
        5 => "a",
        6 => "b",
        _ => unreachable!(),
    }
}

/// Absolute pitch as (pitch_class 0–6, octave) where octave 1 = c' in LilyPond.
/// Middle C (c') = (0, 1).
#[derive(Clone, Copy, Debug)]
struct AbsPitch {
    pc: i32,   // 0=c, 1=d, ..., 6=b
    octave: i32, // LilyPond octave: 0 = c (no mark), 1 = c', -1 = c,
}

impl AbsPitch {
    /// Parse a LilyPond pitch like `c'`, `gis,`, `bes''` into an absolute pitch.
    /// The accidentals (is/es) are ignored for pitch tracking since they don't
    /// affect the relative algorithm's octave placement.
    fn parse(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let first = chars.next()?;
        if !('a'..='g').contains(&first) {
            return None;
        }
        let pc = note_name_to_pitch_class(&first.to_string())?;
        let rest: String = chars.collect();
        // Skip accidentals (is, es, isis, eses)
        let rest = rest
            .trim_start_matches("isis")
            .trim_start_matches("eses")
            .trim_start_matches("is")
            .trim_start_matches("es");
        let ups = rest.matches('\'').count() as i32;
        let downs = rest.matches(',').count() as i32;
        Some(AbsPitch { pc, octave: ups - downs })
    }

    /// Given the previous absolute pitch, resolve a relative note token to
    /// its absolute pitch. LilyPond places the note in the octave closest to
    /// prev (within a fourth), then applies explicit octave marks.
    fn resolve_relative(prev: AbsPitch, token: &str) -> Option<AbsPitch> {
        let mut chars = token.chars().peekable();
        let first = *chars.peek()?;
        if !('a'..='g').contains(&first) {
            return None;
        }
        chars.next();
        let name = first.to_string();
        let pc = note_name_to_pitch_class(&name)?;

        // Consume accidentals
        let rest: String = chars.collect();
        let rest = rest
            .trim_start_matches("isis")
            .trim_start_matches("eses")
            .trim_start_matches("is")
            .trim_start_matches("es");
        let ups = rest.matches('\'').count() as i32;
        let downs = rest.matches(',').count() as i32;

        // Closest octave: compute the interval in pitch classes
        let diff = pc - prev.pc; // -6 to +6
        // Place in closest octave (within a fourth = 3 steps)
        let octave = if diff > 3 {
            prev.octave - 1
        } else if diff < -3 {
            prev.octave + 1
        } else {
            prev.octave
        };

        Some(AbsPitch { pc, octave: octave + ups - downs })
    }

    /// Format as a LilyPond absolute pitch string (e.g., "c'", "g,", "bes''").
    fn to_ly_string(self) -> String {
        let name = pitch_class_to_name(self.pc);
        let marks = if self.octave > 0 {
            "'".repeat(self.octave as usize)
        } else if self.octave < 0 {
            ",".repeat((-self.octave) as usize)
        } else {
            String::new()
        };
        format!("{name}{marks}")
    }
}

/// Track the absolute pitch through a sequence of LilyPond notes in relative mode.
/// Returns the absolute pitch after the last note in the given content.
fn track_pitch(content: &str, start: AbsPitch) -> AbsPitch {
    // Strip LilyPond commands (e.g. \break, \bar, \clef) so their letters
    // aren't mistaken for note names.
    let re_cmd = Regex::new(r"\\[a-zA-Z]+").unwrap();
    let cleaned = re_cmd.replace_all(content, " ");
    // Also strip quoted strings like "|."
    let re_str = Regex::new(r#""[^"]*""#).unwrap();
    let cleaned = re_str.replace_all(&cleaned, " ");
    // Strip comments
    let re_comment = Regex::new(r"%.*").unwrap();
    let cleaned = re_comment.replace_all(&cleaned, " ");

    let re = Regex::new(r"[a-g](is|es|isis|eses)?[',]*").unwrap();
    let mut current = start;
    for m in re.find_iter(&cleaned) {
        let token = m.as_str();
        if let Some(resolved) = AbsPitch::resolve_relative(current, token) {
            current = resolved;
        }
    }
    current
}

/// Split notes content into `n_parts` groups at `\break` boundaries.
/// Returns a Vec of complete LilyPond note blocks, each with the correct
/// `\relative` starting pitch.
fn split_notes(raw_notes: &str, n_parts: usize) -> Vec<String> {
    if n_parts <= 1 {
        return vec![raw_notes.to_string()];
    }

    // Extract preamble (everything up to and including the opening brace line)
    // and the body lines
    let mut preamble_lines = Vec::new();
    let mut body = String::new();
    let mut in_body = false;
    let mut relative_pitch = "c'".to_string();

    for line in raw_notes.lines() {
        if !in_body {
            preamble_lines.push(line.to_string());
            // Extract the relative pitch from the declaration
            if let Some(pos) = line.find("\\relative") {
                let after = &line[pos + "\\relative".len()..];
                let trimmed = after.trim();
                // Parse pitch up to the opening brace
                let pitch_str: String = trimmed.chars()
                    .take_while(|c| *c != '{' && !c.is_whitespace() || *c == '\'' || *c == ',')
                    .collect();
                let pitch_str = pitch_str.trim();
                if !pitch_str.is_empty() {
                    relative_pitch = pitch_str.to_string();
                }
            }
            if line.contains('{') {
                in_body = true;
            }
        } else {
            // Skip closing brace
            if line.trim() == "}" {
                continue;
            }
            // Collect setup commands (\clef, \key, etc.) that appear before any notes
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && (trimmed.starts_with("\\clef")
                    || trimmed.starts_with("\\key")
                    || trimmed.starts_with("\\cadenzaOn")
                    || trimmed.starts_with("\\omit"))
            {
                preamble_lines.push(line.to_string());
                continue;
            }
            body.push_str(line);
            body.push('\n');
        }
    }

    // Split body at \break boundaries into lines
    let mut note_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') {
            // Keep comments/blanks with the current line
            if !current_line.is_empty() {
                current_line.push('\n');
            }
            current_line.push_str(line);
            continue;
        }
        if !current_line.is_empty() {
            current_line.push('\n');
        }
        current_line.push_str(line);
        if trimmed.contains("\\break") || trimmed.contains("\\bar") {
            note_lines.push(current_line.clone());
            current_line.clear();
        }
    }
    if !current_line.trim().is_empty() {
        note_lines.push(current_line);
    }

    let total_lines = note_lines.len();
    let lines_per_part = (total_lines + n_parts - 1) / n_parts;

    // Build the preamble template (everything except the \relative line)
    let mut setup_lines = Vec::new();
    for line in &preamble_lines {
        if !line.contains("\\relative") && !line.contains("melody") {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed != "{" {
                setup_lines.push(trimmed.to_string());
            }
        }
    }

    let start_pitch = AbsPitch::parse(&relative_pitch)
        .unwrap_or(AbsPitch { pc: 0, octave: 1 });
    let mut current_pitch = start_pitch;
    let mut parts = Vec::new();

    for part_idx in 0..n_parts {
        let start = part_idx * lines_per_part;
        let end = ((part_idx + 1) * lines_per_part).min(total_lines);
        if start >= total_lines {
            break;
        }

        let pitch_str = if part_idx == 0 {
            relative_pitch.clone()
        } else {
            current_pitch.to_ly_string()
        };

        let mut part_body = String::new();
        for (i, line_idx) in (start..end).enumerate() {
            let line = &note_lines[line_idx];
            // For non-last lines in the part, keep \break
            // For the last line, replace \break with \bar "|." if it's the last part,
            // or add \bar "|." if not already there
            if i == end - start - 1 {
                // Last line of this part
                let mut l = line.clone();
                if part_idx < n_parts - 1 {
                    // Not the final part: just remove the \break, no barline
                    l = l.replace("\\break", "");
                }
                part_body.push_str(&l);
            } else {
                part_body.push_str(line);
            }
            part_body.push('\n');

            // Track pitch through this line
            current_pitch = track_pitch(line, current_pitch);
        }

        let setup = setup_lines.join("\n  ");
        let part = format!(
            "melody = \\relative {pitch_str} {{\n  {setup}\n\n{part_body}}}\n"
        );
        parts.push(part);
    }

    parts
}

/// Split lyrics content into `n_parts` groups, matching the note line split.
/// Each note line corresponds to one lyrics line.
fn split_lyrics(raw_lyrics: &str, n_parts: usize, total_note_lines: usize) -> Vec<String> {
    if n_parts <= 1 {
        return vec![raw_lyrics.to_string()];
    }

    // Extract lyrics lines (between \lyricmode { and })
    let mut preamble = String::new();
    let mut lyric_lines: Vec<String> = Vec::new();
    let mut in_body = false;

    for line in raw_lyrics.lines() {
        if !in_body {
            if line.contains("\\lyricmode") || line.contains("{") {
                preamble = line.split('{').next().unwrap_or("verse = \\lyricmode").to_string();
                // Check if there's content after the brace on this line
                if let Some(after) = line.split('{').nth(1) {
                    let content = after.trim().trim_end_matches('}').trim();
                    if !content.is_empty() {
                        lyric_lines.push(content.to_string());
                    }
                }
                in_body = true;
            }
        } else {
            let trimmed = line.trim();
            if trimmed == "}" {
                break;
            }
            if !trimmed.is_empty() {
                lyric_lines.push(trimmed.to_string());
            }
        }
    }

    let lines_per_part = (total_note_lines + n_parts - 1) / n_parts;
    let mut parts = Vec::new();

    for part_idx in 0..n_parts {
        let start = part_idx * lines_per_part;
        let end = ((part_idx + 1) * lines_per_part).min(lyric_lines.len());
        if start >= lyric_lines.len() {
            parts.push(format!("{preamble} {{\n}}\n"));
            continue;
        }

        let body: String = lyric_lines[start..end]
            .iter()
            .map(|l| format!("  {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("{preamble} {{\n{body}\n}}\n"));
    }

    parts
}

/// Count the number of note lines (segments separated by \break or \bar) in raw notes.
fn count_note_lines(raw_notes: &str) -> usize {
    let breaks = raw_notes.matches("\\break").count();
    let bars = raw_notes.matches("\\bar").count();
    breaks + bars
}

/// Apply visual tweaks to note content before rendering:
/// - Hide clef after the first line break
/// - If `force_combine` or enough lines, combine pairs by removing odd `\break`s
/// - Add invisible rests at line boundaries for alignment
fn modify_notes(notes: &str, force_combine: bool) -> String {
    let re_note = Regex::new(r"[a-g](is|es)?[0-9]").unwrap();
    let re_rest_end = Regex::new(r"r[0-9]+\.?\s*(\\break|\\bar)").unwrap();
    let re_break_bar = Regex::new(r"(\s*\\break|\s*\\bar)").unwrap();

    // If forced, combine pairs by removing \break on odd-numbered lines
    let combined = force_combine;
    let notes = {
        if combined {
            let mut result = String::new();
            let mut break_idx = 0usize;
            let mut rest: &str = notes.as_ref();
            while let Some(pos) = rest.find("\\break") {
                break_idx += 1;
                result.push_str(&rest[..pos]);
                if break_idx % 2 == 1 {
                    // odd-numbered break: skip it
                } else {
                    result.push_str("\\break");
                }
                rest = &rest[pos + "\\break".len()..];
            }
            result.push_str(rest);
            result
        } else {
            notes.to_string()
        }
    };

    let notes = notes.replacen("\\break", "\\break\n  \\omit Staff.Clef", 1);

    // Track which "original" note line we're on so that in combined mode
    // we only add the starting hidden rest on odd lines (first of each pair).
    let mut note_line_idx = 0usize;
    notes
        .lines()
        .map(|line| {
            let stripped = line.trim();
            if stripped.starts_with('%')
                || stripped.starts_with('\\')
                || stripped.is_empty()
                || stripped == "}"
                || stripped.contains("melody")
                || stripped.contains('=')
                || !re_note.is_match(stripped)
            {
                return line.to_string();
            }

            let is_odd = note_line_idx % 2 == 0; // 0-indexed: 0,2,4 are odd original lines (1st,3rd,5th)
            note_line_idx += 1;

            let mut out = line.to_string();
            // Add hidden rest at start only for odd lines (or all lines if not combining)
            if !stripped.starts_with('r') && (!combined || is_odd) {
                out = out.replacen(stripped, &format!("\\once \\hide Rest r4 {stripped}"), 1);
            }
            if !re_rest_end.is_match(out.trim()) {
                out = re_break_bar
                    .replacen(&out, 1, " \\once \\hide Rest r2$1")
                    .to_string();
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Strip or replace LaTeX-style markup from lyrics content so it can be safely
/// used in LilyPond's `\lyricmode`.
fn sanitize_lyrics(content: &str) -> String {
    // Replace escaped straight quotes with curly quotes (safe in lyricmode)
    let s = content.replace("\\\"", "\u{201c}");
    let s = Regex::new(r"\\(left|right|textit|textbf|emph)\s*")
        .unwrap()
        .replace_all(&s, "");
    Regex::new(r"\\u[0-9a-fA-F]{4}")
        .unwrap()
        .replace_all(&s, "")
        .to_string()
}

/// Count the maximum number of note/rest events across all lines in the raw notes content.
/// Lines are delimited by `\break` or `\bar`. Returns 8 as a fallback minimum.
fn max_notes_per_line(raw_notes: &str) -> usize {
    let re = Regex::new(r"[a-gr](is|es)?[',]*\d").unwrap();
    raw_notes
        .split("\\break")
        .flat_map(|s| s.split("\\bar"))
        .map(|segment| {
            let content: String = segment
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with('%') && !t.starts_with("\\omit")
                })
                .collect::<Vec<_>>()
                .join(" ");
            re.find_iter(&content).count()
        })
        .max()
        .unwrap_or(8)
        .max(8)
}

/// Assemble a complete LilyPond `.ly` file from notes, lyrics, and an optional
/// composer credit, ready for rendering.
fn build_combined_ly(notes: &str, lyrics: &str, composer: Option<&str>, paper_width_mm: usize) -> String {
    let mut header_items = Vec::new();
    if let Some(c) = composer {
        header_items.push(format!("  composer = \"{c}\""));
    }
    header_items.push("  tagline = ##f".into());
    let header = format!("\\header {{\n{}\n}}", header_items.join("\n"));

    let lyrics_score = if lyrics.trim().is_empty() {
        String::new()
    } else {
        "    \\new Lyrics \\lyricsto \"melody\" { \\verse }".into()
    };

    // If there are no standalone | barline separators, inject \accidentalStyle forget
    // so accidentals are printed on every note (the whole piece is one long measure).
    let has_barlines = notes.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('%') && trimmed.split_whitespace().any(|tok| tok == "|")
    });
    let notes = if has_barlines {
        notes.to_string()
    } else {
        notes.replacen("\\cadenzaOn", "\\cadenzaOn\n  \\accidentalStyle forget", 1)
    };

    format!(
        r#"\version "2.24.0"

\paper {{
  paper-width = {paper_width_mm}\mm
  line-width = {paper_width_mm}\mm
  left-margin = 0\cm
  right-margin = 0\cm
}}

{header}

{notes}

{lyrics}

\score {{
  <<
    \new Voice = "melody" {{ \melody }}
{lyrics_score}
  >>
  \layout {{
    indent = 0
    \context {{
      \Score
      \override SpacingSpanner.uniform-stretching = ##t
      \override SpacingSpanner.strict-note-spacing = ##t
    }}
    \context {{
      \Lyrics
      \override LyricText.self-alignment-X = #CENTER
    }}
  }}
}}
"#
    )
}

/// Determine the effective number of slide parts for a song based on its
/// split style and the number of `\break`s in its notes.
fn effective_n_parts(split_style: &SplitStyle, break_count: usize) -> usize {
    match split_style {
        SplitStyle::Default | SplitStyle::MultiSlide => {
            let total_lines = break_count + 1;
            (total_lines + COMBINE_LINES_THRESHOLD - 1) / COMBINE_LINES_THRESHOLD
        }
        SplitStyle::SingleSlide | SplitStyle::CombineLines => 1,
    }
}

/// Build the combined .ly content parts for a verse.
/// Returns empty Vec if notes.ly doesn't exist.
fn build_combined_parts_for_verse(song_dir: &Path, verse: u32) -> Vec<String> {
    let notes_file = song_dir.join("notes.ly");
    let lyrics = song_dir.join(format!("lyrics_{verse}.ly"));

    if !notes_file.exists() {
        return Vec::new();
    }

    let raw_notes = match fs::read_to_string(&notes_file) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let meta = read_song_meta(song_dir);
    let break_count = raw_notes.matches("\\break").count();
    let n_parts = effective_n_parts(&meta.split_style, break_count);

    let raw_lyrics = if lyrics.exists() {
        sanitize_lyrics(&fs::read_to_string(&lyrics).unwrap_or_default())
    } else {
        String::new()
    };

    if n_parts > 1 {
        let total_lines = count_note_lines(&raw_notes);
        let note_parts = split_notes(&raw_notes, n_parts);
        let lyric_parts = split_lyrics(&raw_lyrics, n_parts, total_lines);

        note_parts.iter().zip(lyric_parts.iter()).map(|(notes, lyrics)| {
            let modified = modify_notes(notes, false);
            let paper_width_mm = max_notes_per_line(&modified) * 9 + 20;
            // Only show composer on the first part
            let comp = if notes == &note_parts[0] { meta.composer.as_deref() } else { None };
            build_combined_ly(&modified, lyrics, comp, paper_width_mm)
        }).collect()
    } else {
        // Single part: apply modify_notes (combining / hidden rests)
        let force_combine = meta.split_style == SplitStyle::CombineLines;
        let notes_content = modify_notes(&raw_notes, force_combine);
        let paper_width_mm = max_notes_per_line(&notes_content) * 9 + 20;
        vec![build_combined_ly(&notes_content, &raw_lyrics, meta.composer.as_deref(), paper_width_mm)]
    }
}

/// Return the number of parts (slides) needed for a verse.
pub fn num_parts_for_verse(song_dir: &Path, _verse: u32) -> usize {
    let notes_file = song_dir.join("notes.ly");
    if let Ok(raw_notes) = fs::read_to_string(&notes_file) {
        let meta = read_song_meta(song_dir);
        let break_count = raw_notes.matches("\\break").count();
        effective_n_parts(&meta.split_style, break_count)
    } else {
        1
    }
}

/// Check whether a cached SVG exists for the current source content.
pub fn is_svg_current(song_dir: &Path, verse: u32, part: u32) -> bool {
    let parts = build_combined_parts_for_verse(song_dir, verse);
    if let Some(combined) = parts.get(part as usize) {
        cached_svg_path(combined).exists()
    } else {
        false
    }
}

/// Return the cached SVG path for a verse part if it exists, for loading.
pub fn svg_path_for_verse(song_dir: &Path, verse: u32, part: u32) -> Option<PathBuf> {
    let parts = build_combined_parts_for_verse(song_dir, verse);
    let combined = parts.get(part as usize)?;
    let path = cached_svg_path(combined);
    if path.exists() { Some(path) } else { None }
}

/// Render the SVG for a given verse part. Returns `Ok(())` on success,
/// `Err(message)` with the LilyPond error output on failure.
/// This function is safe to call from a background thread.
pub fn render_svg(song_dir: &Path, verse: u32, part: u32) -> Result<(), String> {
    let notes = song_dir.join("notes.ly");
    if !notes.exists() {
        return Err("notes.ly not found".into());
    }

    let parts = build_combined_parts_for_verse(song_dir, verse);
    let combined = parts.get(part as usize)
        .ok_or("Failed to build combined .ly")?;
    let svg_out = cached_svg_path(combined);

    // Already cached
    if svg_out.exists() {
        return Ok(());
    }

    let dir_name = song_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    eprintln!("Rendering {dir_name}/{verse} part {part}...");

    let cache_dir = svg_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {e}"))?;

    let hash = content_hash(combined);
    let combined_ly = cache_dir.join(format!("_combined_{hash}.ly"));
    eprintln!("Writing combined .ly to {}", combined_ly.display());
    fs::write(&combined_ly, combined)
        .map_err(|e| format!("Failed to write combined .ly: {e}"))?;

    let stem = svg_out.with_extension("");
    #[allow(unused_mut)]
    let mut cmd = Command::new(lilypond_bin());
    cmd.args(["-dbackend=svg", "-dcrop", "-o"])
        .arg(&stem)
        .arg(&combined_ly)
        .current_dir(&cache_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let result = cmd.output();

    match result {
        Ok(out) if out.status.success() => {
            // LilyPond with -dcrop produces a .cropped.svg alongside the main SVG
            let cropped = stem.with_extension("cropped.svg");
            if cropped.exists() {
                let _ = fs::rename(&cropped, &svg_out);
            }
            // let _ = fs::remove_file(&combined_ly);
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            eprintln!("lilypond failed for {dir_name}/{verse} part {part}: {stderr}");
            // let _ = fs::remove_file(&combined_ly);
            Err(stderr)
        }
        Err(e) => {
            let msg = format!("Failed to run lilypond: {e}");
            eprintln!("{msg}");
            Err(msg)
        }
    }
}
