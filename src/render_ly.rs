//! On-demand LilyPond SVG rendering.
//!
//! Combines `notes.ly` + `lyrics_N.ly` + `composer.txt` into a single
//! `.ly` file and invokes `lilypond` to produce an SVG. Skips rendering
//! when the output SVG is already newer than all source files.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Find the lilypond binary: check next to our executable first, then PATH.
fn lilypond_bin() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Check for bundled lilypond: <exe_dir>/lilypond-bin/bin/lilypond
            let bundled = dir.join("lilypond-bin").join("bin").join("lilypond");
            if bundled.exists() {
                return bundled;
            }
        }
    }
    PathBuf::from("lilypond")
}

fn is_up_to_date(output: &Path, sources: &[&Path]) -> bool {
    let out_mtime = match fs::metadata(output).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    sources.iter().all(|src| {
        fs::metadata(src)
            .and_then(|m| m.modified())
            .map(|t| t <= out_mtime)
            .unwrap_or(false)
    })
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

fn sanitize_lyrics(content: &str) -> String {
    let s = content
        .replace('\u{201c}', "\"")
        .replace('\u{201d}', "\"")
        .replace('\u{2018}', "'")
        .replace('\u{2019}', "'");
    let s = Regex::new(r"\\(left|right|textit|textbf|emph)\s*")
        .unwrap()
        .replace_all(&s, "");
    Regex::new(r"\\u[0-9a-fA-F]{4}")
        .unwrap()
        .replace_all(&s, "")
        .to_string()
}

fn build_combined_ly(notes: &str, lyrics: &str, composer: Option<&str>) -> String {
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
  line-width = 13\cm
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
      \override LyricText.self-alignment-X = #LEFT
    }}
  }}
}}
"#
    )
}

/// Ensure the SVG for a given verse exists and is up to date.
///
/// Returns `true` if the SVG is available after this call.
pub fn ensure_svg(song_dir: &Path, verse: u32) -> bool {
    let svg = song_dir.join(format!("{verse}.svg"));
    let notes = song_dir.join("notes.ly");
    let lyrics = song_dir.join(format!("lyrics_{verse}.ly"));
    let composer_file = song_dir.join("composer.txt");

    if !notes.exists() {
        return svg.exists();
    }

    let mut sources: Vec<&Path> = vec![&notes];
    if lyrics.exists() {
        sources.push(&lyrics);
    }
    if composer_file.exists() {
        sources.push(&composer_file);
    }
    if is_up_to_date(&svg, &sources) {
        return true;
    }

    let dir_name = song_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    eprintln!("Rendering {dir_name}/{verse}...");

    let notes_content = match fs::read_to_string(&notes) {
        Ok(s) => modify_notes(&s),
        Err(_) => return false,
    };
    let lyrics_content = if lyrics.exists() {
        sanitize_lyrics(&fs::read_to_string(&lyrics).unwrap_or_default())
    } else {
        String::new()
    };
    let composer = fs::read_to_string(&composer_file)
        .ok()
        .map(|s| s.trim().to_string());

    let combined = build_combined_ly(
        &notes_content,
        &lyrics_content,
        composer.as_deref(),
    );

    let combined_ly = song_dir.join(format!("_combined_{verse}.ly"));
    if fs::write(&combined_ly, &combined).is_err() {
        return false;
    }

    let stem = song_dir.join(format!("{verse}"));
    let result = Command::new(lilypond_bin())
        .args(["-dbackend=svg", "-dcrop", "-o"])
        .arg(&stem)
        .arg(&combined_ly)
        .current_dir(song_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    match result {
        Ok(out) if out.status.success() => {
            let cropped = song_dir.join(format!("{verse}.cropped.svg"));
            if cropped.exists() {
                let _ = fs::rename(&cropped, &svg);
            }
            let _ = fs::remove_file(&combined_ly);
            true
        }
        Ok(out) => {
            eprintln!(
                "lilypond failed for {dir_name}/{verse}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            false
        }
        Err(e) => {
            eprintln!("Failed to run lilypond: {e}");
            false
        }
    }
}
