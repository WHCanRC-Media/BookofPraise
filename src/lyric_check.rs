//! Post-render verification of lyric text against source.
//!
//! Parses a rendered LilyPond SVG, reconstructs the displayed lyric lines
//! in melody order (merging hyphen-separated syllables), and compares each
//! line against `lyrics/<song>.txt` under a lenient normalization.
//!
//! Accounts for `split_style: combine lines`, which renders pairs of source
//! lines as a single display line, and for multi-part songs, which split
//! source lines across slides.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use crate::render_ly::{read_song_meta, SplitStyle};

/// Keys already logged, to avoid spamming stderr when the same verse-part is
/// re-checked (e.g. every time a slide is shown).
static LOGGED: LazyLock<Mutex<HashSet<(PathBuf, u32, u32)>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static RE_G: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<g transform="translate\(([-0-9.]+),\s*([-0-9.]+)\)"\s*>(.*?)</g>"#).unwrap()
});
static RE_TSPAN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<tspan[^>]*>([^<]*)</tspan>"#).unwrap());
static RE_VERSE_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\.\s*$").unwrap());

#[derive(Debug, Clone)]
struct TextItem {
    y: f64,
    x: f64,
    text: String,
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

fn parse_svg_text(svg: &str) -> Vec<TextItem> {
    let mut items = Vec::new();
    for caps in RE_G.captures_iter(svg) {
        let x: f64 = caps[1].parse().unwrap_or(0.0);
        let y: f64 = caps[2].parse().unwrap_or(0.0);
        let inner = &caps[3];
        for tcaps in RE_TSPAN.captures_iter(inner) {
            let text = decode_entities(&tcaps[1]);
            if !text.is_empty() {
                items.push(TextItem { y, x, text });
            }
        }
    }
    items
}

fn is_lyric(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty() && !trimmed.chars().all(|c| c.is_ascii_digit())
}

fn group_into_lines(items: Vec<TextItem>) -> Vec<Vec<(f64, String)>> {
    let mut sorted: Vec<TextItem> = items.into_iter().filter(|i| is_lyric(&i.text)).collect();
    sorted.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let y_tol = 0.8;
    let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    for item in sorted {
        match lines.last_mut() {
            Some((ly, entries)) if (item.y - *ly).abs() <= y_tol => {
                entries.push((item.x, item.text));
            }
            _ => lines.push((item.y, vec![(item.x, item.text)])),
        }
    }
    for (_, entries) in &mut lines {
        entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }
    lines.into_iter().map(|(_, entries)| entries).collect()
}

fn join_syllables(line: &[(f64, String)]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut pending_join = false;
    for (_, tok) in line {
        let t = tok.as_str();
        if matches!(t, "-" | "\u{2013}" | "\u{2014}") {
            pending_join = true;
            continue;
        }
        if pending_join {
            if let Some(last) = out.last_mut() {
                last.push_str(t);
            } else {
                out.push(t.to_string());
            }
        } else {
            out.push(t.to_string());
        }
        pending_join = false;
    }
    out.join(" ")
}

/// Extract displayed lyric lines from a rendered SVG in melody order.
pub fn extract_svg_lyrics(svg_path: &Path) -> Vec<String> {
    let Ok(content) = fs::read_to_string(svg_path) else {
        return Vec::new();
    };
    let items = parse_svg_text(&content);
    let lines = group_into_lines(items);
    lines
        .iter()
        .map(|l| join_syllables(l))
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Lenient text comparison: lowercase, keep only alphanumerics.
fn normalize(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Parse a `lyrics/<song>.txt` file into `{verse -> lines}`.
pub fn load_source_lyrics(path: &Path) -> HashMap<u32, Vec<String>> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut verses: HashMap<u32, Vec<String>> = HashMap::new();
    let mut current: Option<u32> = None;
    for line in content.lines() {
        let line = line.trim_end();
        if let Some(caps) = RE_VERSE_HEADER.captures(line) {
            current = caps[1].parse().ok();
        } else if let Some(v) = current {
            if !line.trim().is_empty() {
                verses.entry(v).or_default().push(line.to_string());
            }
        }
    }
    verses
}

/// Locate the repo-root `lyrics/<song>.txt` file corresponding to a song dir
/// like `.../lilypond/hymn1`.
fn source_lyrics_path(song_dir: &Path) -> Option<PathBuf> {
    let name = song_dir.file_name()?.to_str()?;
    let repo = song_dir.parent()?.parent()?;
    Some(repo.join("lyrics").join(format!("{name}.txt")))
}

/// Build the slice of source lines that should appear on the given part.
///
/// - `CombineLines`: pair adjacent lines with a space; one part only.
/// - Multi-part (n_parts > 1): take this part's contiguous slice.
/// - Otherwise: the full verse.
fn source_slice_for_part(
    src_lines: &[String],
    split_style: &SplitStyle,
    n_parts: usize,
    part: usize,
) -> Vec<String> {
    if *split_style == SplitStyle::CombineLines {
        let mut out = Vec::new();
        let mut i = 0;
        while i < src_lines.len() {
            if i + 1 < src_lines.len() {
                out.push(format!("{} {}", src_lines[i], src_lines[i + 1]));
                i += 2;
            } else {
                out.push(src_lines[i].clone());
                i += 1;
            }
        }
        return out;
    }
    if n_parts > 1 {
        let lines_per_part = (src_lines.len() + n_parts - 1) / n_parts;
        let start = (part * lines_per_part).min(src_lines.len());
        let end = ((part + 1) * lines_per_part).min(src_lines.len());
        return src_lines[start..end].to_vec();
    }
    src_lines.to_vec()
}

/// Compare rendered SVG lyrics for one verse-part against source lyrics.
/// Returns a list of human-readable mismatch lines (empty when everything matches).
pub fn compare_svg_to_source(
    song_dir: &Path,
    verse: u32,
    part: u32,
    svg_path: &Path,
    n_parts: usize,
) -> Vec<String> {
    let Some(src_path) = source_lyrics_path(song_dir) else {
        return Vec::new();
    };
    if !src_path.exists() {
        return Vec::new();
    }

    let src_all = load_source_lyrics(&src_path);
    let Some(src_lines) = src_all.get(&verse) else {
        return vec![format!(
            "source verse {verse} missing in {}",
            src_path.display()
        )];
    };

    let meta = read_song_meta(song_dir);
    let expected = source_slice_for_part(src_lines, &meta.split_style, n_parts, part as usize);
    let displayed = extract_svg_lyrics(svg_path);

    let mut mismatches = Vec::new();
    let max_len = displayed.len().max(expected.len());
    for i in 0..max_len {
        let src = expected.get(i).cloned().unwrap_or_default();
        let svg = displayed.get(i).cloned().unwrap_or_default();
        if normalize(&src) != normalize(&svg) {
            mismatches.push(format!("  line {}:", i + 1));
            mismatches.push(format!("    src: {src}"));
            mismatches.push(format!("    svg: {svg}"));
        }
    }
    mismatches
}

/// Collect (src, svg) pairs for lines that differ, with ASCII spaces removed
/// from both strings.
pub fn mismatch_pairs_nospace(
    song_dir: &Path,
    verse: u32,
    part: u32,
    svg_path: &Path,
    n_parts: usize,
) -> Vec<(String, String)> {
    let Some(src_path) = source_lyrics_path(song_dir) else {
        return Vec::new();
    };
    if !src_path.exists() {
        return Vec::new();
    }
    let src_all = load_source_lyrics(&src_path);
    let Some(src_lines) = src_all.get(&verse) else {
        return Vec::new();
    };
    let meta = read_song_meta(song_dir);
    let expected = source_slice_for_part(src_lines, &meta.split_style, n_parts, part as usize);
    let displayed = extract_svg_lyrics(svg_path);

    let strip = |s: &str| -> String { s.chars().filter(|c| *c != ' ').collect() };

    let mut out = Vec::new();
    let max_len = displayed.len().max(expected.len());
    for i in 0..max_len {
        let src = expected.get(i).cloned().unwrap_or_default();
        let svg = displayed.get(i).cloned().unwrap_or_default();
        if normalize(&src) != normalize(&svg) {
            out.push((strip(&src), strip(&svg)));
        }
    }
    out
}

/// Compare, log mismatches to stderr once per (song, verse, part), and return
/// the (src, svg) pairs (spaces removed) for on-slide display.
pub fn check_and_log(
    song_dir: &Path,
    verse: u32,
    part: u32,
    svg_path: &Path,
    n_parts: usize,
) -> Vec<(String, String)> {
    let diffs = compare_svg_to_source(song_dir, verse, part, svg_path, n_parts);
    if diffs.is_empty() {
        return Vec::new();
    }
    let key = (song_dir.to_path_buf(), verse, part);
    let should_log = match LOGGED.lock() {
        Ok(mut set) => set.insert(key),
        Err(_) => true,
    };
    if should_log {
        let name = song_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        eprintln!("lyric mismatches in {name}/verse {verse} part {part}:");
        for d in diffs {
            eprintln!("{d}");
        }
    }
    mismatch_pairs_nospace(song_dir, verse, part, svg_path, n_parts)
}
