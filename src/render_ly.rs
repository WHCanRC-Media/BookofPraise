//! On-demand LilyPond SVG rendering.
//!
//! Combines `notes.ly` + `lyrics_N.ly` + `composer.txt` into a single
//! `.ly` file and invokes `lilypond` to produce an SVG. Skips rendering
//! when the output SVG is already newer than all source files.

use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Return the platform cache directory for rendered SVGs.
pub fn svg_cache_dir() -> PathBuf {
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
    base.join("bop").join("svg")
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

/// Find the lilypond binary: check next to our executable first, then PATH.
fn lilypond_bin() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Check for bundled lilypond: <exe_dir>/lilypond-bin/bin/lilypond
            let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
            let bundled = dir.join("lilypond-bin").join("bin").join(format!("lilypond{exe_suffix}"));
            if bundled.exists() {
                return bundled;
            }
        }
    }
    PathBuf::from("lilypond")
}

/// Apply visual tweaks to note content before rendering:
/// - Hide clef after the first line break
/// - Add invisible rests at line boundaries for alignment
fn modify_notes(notes: &str) -> String {
    let re_note = Regex::new(r"[a-g](is|es)?[0-9]").unwrap();
    let re_rest_end = Regex::new(r"r[12]\s*(\\break|\\bar)").unwrap();
    let re_break_bar = Regex::new(r"(\s*\\break|\s*\\bar)").unwrap();

    let notes = notes.replacen("\\break", "\\break\n  \\omit Staff.Clef", 1);

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

            let mut out = line.to_string();
            if !stripped.starts_with('r') {
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
fn build_combined_ly(notes: &str, lyrics: &str, composer: Option<&str>, paper_width: usize) -> String {
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

    format!(
        r#"\version "2.24.0"

\paper {{
  paper-width = {paper_width}\cm
  line-width = {paper_width}\cm
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
      \Lyrics
      \override LyricText.self-alignment-X = #CENTER
    }}
  }}
}}
"#
    )
}

/// Build the combined .ly content for a verse (without rendering).
/// Returns `None` if notes.ly doesn't exist.
fn build_combined_for_verse(song_dir: &Path, verse: u32) -> Option<String> {
    let notes = song_dir.join("notes.ly");
    let lyrics = song_dir.join(format!("lyrics_{verse}.ly"));
    let composer_file = song_dir.join("composer.txt");

    if !notes.exists() {
        return None;
    }

    let raw_notes = fs::read_to_string(&notes).ok()?;
    let paper_width = max_notes_per_line(&raw_notes) + 2;
    let notes_content = modify_notes(&raw_notes);
    let lyrics_content = if lyrics.exists() {
        sanitize_lyrics(&fs::read_to_string(&lyrics).unwrap_or_default())
    } else {
        String::new()
    };
    let composer = fs::read_to_string(&composer_file)
        .ok()
        .map(|s| s.trim().to_string());

    Some(build_combined_ly(&notes_content, &lyrics_content, composer.as_deref(), paper_width))
}

/// Check whether a cached SVG exists for the current source content.
pub fn is_svg_current(song_dir: &Path, verse: u32) -> bool {
    match build_combined_for_verse(song_dir, verse) {
        Some(combined) => cached_svg_path(&combined).exists(),
        None => false,
    }
}

/// Return the cached SVG path for a verse if it exists, for loading.
pub fn svg_path_for_verse(song_dir: &Path, verse: u32) -> Option<PathBuf> {
    let combined = build_combined_for_verse(song_dir, verse)?;
    let path = cached_svg_path(&combined);
    if path.exists() { Some(path) } else { None }
}

/// Render the SVG for a given verse. Returns `Ok(PathBuf)` with the cached
/// SVG path on success, `Err(message)` with the LilyPond error output on failure.
/// This function is safe to call from a background thread.
pub fn render_svg(song_dir: &Path, verse: u32) -> Result<(), String> {
    let notes = song_dir.join("notes.ly");
    if !notes.exists() {
        return Err("notes.ly not found".into());
    }

    let combined = build_combined_for_verse(song_dir, verse)
        .ok_or("Failed to build combined .ly")?;
    let svg_out = cached_svg_path(&combined);

    // Already cached
    if svg_out.exists() {
        return Ok(());
    }

    let dir_name = song_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    eprintln!("Rendering {dir_name}/{verse}...");

    let cache_dir = svg_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {e}"))?;

    let hash = content_hash(&combined);
    let combined_ly = cache_dir.join(format!("_combined_{hash}.ly"));
    fs::write(&combined_ly, &combined)
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
            let _ = fs::remove_file(&combined_ly);
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            eprintln!("lilypond failed for {dir_name}/{verse}: {stderr}");
            let _ = fs::remove_file(&combined_ly);
            Err(stderr)
        }
        Err(e) => {
            let msg = format!("Failed to run lilypond: {e}");
            eprintln!("{msg}");
            Err(msg)
        }
    }
}
