use clap::Parser;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use gtk4 as gtk;
use gtk::gdk;

use crate::rendering::DEFAULT_RENDER_WIDTH;

/// When true, `base_dir` resolves to the platform data directory instead of cwd/exe.
static USE_DATA_DIR: AtomicBool = AtomicBool::new(false);

/// Call once at startup when `--update` is set so that `base_dir` and the
/// updater use the platform data directory (`%APPDATA%\bop` / `~/.local/share/bop`).
pub fn enable_data_dir_mode() {
    USE_DATA_DIR.store(true, Ordering::Relaxed);
}

pub fn data_dir_mode() -> bool {
    USE_DATA_DIR.load(Ordering::Relaxed)
}

// ── CLI ─────────────────────────────────────────────────────────────

/// Book of Praise — hymn and psalm display application
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Psalm numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    pub psalm: Vec<u32>,

    /// Hymn numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    pub hymn: Vec<u32>,

    /// Check for updates on startup
    #[arg(long, default_value_t = cfg!(feature = "auto-update"))]
    pub update: bool,

    /// Force rendering every verse onto a single slide, overriding per-song split style.
    #[arg(long)]
    pub force_one_slide: bool,
}

// ── Data model ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum SongType {
    Psalm,
    Hymn,
}

impl SongType {
    /// Return the filesystem prefix for this song type (e.g. `"psalm"` or `"hymn"`).
    pub fn prefix(self) -> &'static str {
        match self {
            SongType::Psalm => "psalm",
            SongType::Hymn => "hymn",
        }
    }

    /// Return a human-readable label like `"Psalm 42"` or `"Hymn 7"`.
    pub fn label(self, num: u32) -> String {
        match self {
            SongType::Psalm => format!("Psalm {num}"),
            SongType::Hymn => format!("Hymn {num}"),
        }
    }
}

pub struct SongLibrary {
    psalms: BTreeMap<u32, Vec<u32>>,
    hymns: BTreeMap<u32, Vec<u32>>,
}

impl SongLibrary {
    /// Scan a directory for psalm and hymn subdirectories, building a mapping
    /// from song number to available verse numbers.
    pub fn scan(dir: &Path) -> Self {
        let mut psalms = BTreeMap::new();
        let mut hymns = BTreeMap::new();

        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let (map, prefix) = if let Some(rest) = name.strip_prefix("psalm") {
                (&mut psalms, rest)
            } else if let Some(rest) = name.strip_prefix("hymn") {
                (&mut hymns, rest)
            } else {
                continue;
            };
            if let Ok(num) = prefix.parse::<u32>() {
                let verses = scan_verses(&entry.path());
                if !verses.is_empty() {
                    map.insert(num, verses);
                }
            }
        }

        SongLibrary { psalms, hymns }
    }

    /// Look up the available verses for a song by type and number.
    pub fn get(&self, song_type: SongType, num: u32) -> Option<&Vec<u32>> {
        match song_type {
            SongType::Psalm => self.psalms.get(&num),
            SongType::Hymn => self.hymns.get(&num),
        }
    }
}

/// Scan a song directory for verse numbers by examining image files and LilyPond sources.
fn scan_verses(dir: &Path) -> Vec<u32> {
    let mut verses = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let stem = Path::new(&name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        // Image files: "1.svg", "2a.png", etc.
        let num_str: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(v) = num_str.parse::<u32>() {
            verses.insert(v);
            continue;
        }
        // LilyPond source: "lyrics_1.ly"
        if let Some(rest) = stem.strip_prefix("lyrics_") {
            if let Ok(v) = rest.parse::<u32>() {
                verses.insert(v);
            }
        }
    }
    verses.into_iter().collect()
}

/// Read the verification count for a specific verse from `song.yaml`.
pub fn read_verify_count(song_dir: &Path, verse: u32) -> u32 {
    let meta = crate::render_ly::read_song_meta(song_dir);
    meta.verified.get(&verse).copied().unwrap_or(0)
}

/// Increment and persist the verification count for a verse, returning the new count.
pub fn increment_verify(song_dir: &Path, verse: u32) -> u32 {
    let mut meta = crate::render_ly::read_song_meta(song_dir);
    let count = meta.verified.get(&verse).copied().unwrap_or(0) + 1;
    meta.verified.insert(verse, count);
    crate::render_ly::write_song_meta(song_dir, &meta);
    count
}

/// Hymns that require usage tracking (e.g. for copyright/licensing reporting).
const TRACKED_HYMNS: &[u32] = &[38, 50, 66, 79];

/// Record usage of a tracked hymn to a log file. Each entry is recorded once
/// per day. The file path is taken from the `HYMN_USAGE_TXT` environment
/// variable, defaulting to `~/Desktop/HymnUsage.txt`.
fn record_hymn_usage(song_type: SongType, num: u32) {
    if song_type != SongType::Hymn || !TRACKED_HYMNS.contains(&num) {
        return;
    }
    let path = std::env::var("HYMN_USAGE_TXT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let dir = crate::render_ly::data_dir();
            let _ = std::fs::create_dir_all(&dir);
            dir.join("HymnUsage.txt")
        });
    let path = path.to_string_lossy().to_string();
    // Format date as Mon-DD-YYYY to match the legacy web app format
    let now = std::time::SystemTime::now();
    let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    // Simple date calculation: days since epoch
    let days = secs / 86400;
    let (year, month, day) = epoch_days_to_date(days);
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let entry = format!("{}-{:02}-{} H{num}", months[month as usize - 1], day, year);
    let contents = std::fs::read_to_string(&path).unwrap_or_default();
    if contents.lines().any(|line| line.trim() == entry) {
        return;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path);
    if let Ok(ref mut f) = file {
        use std::io::Write;
        let _ = writeln!(f, "{entry}");
    }
}

/// Convert days since Unix epoch to (year, month, day).
pub fn epoch_days_to_date(days: u64) -> (u64, u64, u64) {
    // Civil calendar algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[derive(Clone)]
pub struct LiturgyEntry {
    pub song_name: String,
    pub song_dir: String,
    pub verses: Vec<u32>,
}

#[derive(Clone)]
pub struct Slide {
    pub title: String,
    pub all_verses: Vec<u32>,
    pub current_verse: u32,
    pub part: u32,
    pub path: PathBuf,
    pub song_dir: PathBuf,
}

pub type CacheKey = (PathBuf, u32, u32, u32); // (path, verse, part, render_width)

pub struct AppState {
    pub songs_dir: PathBuf,
    pub library: SongLibrary,
    pub liturgy: Vec<LiturgyEntry>,
    pub slides: Vec<Slide>,
    pub current_slide: usize,
    pub render_width: u32,
    pub texture_cache: HashMap<CacheKey, gdk::Texture>,
    pub rendering: HashSet<(PathBuf, u32, u32)>,
    pub render_errors: HashMap<(PathBuf, u32, u32), String>,
    /// Song dirs that had edits saved this session (for email-patch-on-close).
    pub edited_song_dirs: HashSet<PathBuf>,
    /// Temp directory holding original file snapshots (created on first edit).
    pub originals_dir: Option<tempfile::TempDir>,
}

impl AppState {
    /// Initialize application state from CLI arguments. Loads the song library,
    /// adds any songs specified on the command line (defaulting to Psalm 1), and
    /// builds the initial slide list.
    pub fn new(cli: &Cli) -> Self {
        let songs_dir = base_dir();
        let library = SongLibrary::scan(&songs_dir);

        let mut state = AppState {
            songs_dir,
            library,
            liturgy: Vec::new(),
            slides: Vec::new(),
            current_slide: 0,
            render_width: DEFAULT_RENDER_WIDTH,
            texture_cache: HashMap::new(),
            rendering: HashSet::new(),
            render_errors: HashMap::new(),
            edited_song_dirs: HashSet::new(),
            originals_dir: None,
        };

        // Load songs from CLI
        for &n in &cli.psalm {
            state.add_song(SongType::Psalm, n);
        }
        for &n in &cli.hymn {
            state.add_song(SongType::Hymn, n);
        }
        state.rebuild_slides();
        state
    }

    /// Add all verses of a song to the liturgy. Returns `true` if the song was found.
    pub fn add_song(&mut self, song_type: SongType, num: u32) -> bool {
        if let Some(verses) = self.library.get(song_type, num).cloned() {
            record_hymn_usage(song_type, num);
            self.liturgy.push(LiturgyEntry {
                song_name: song_type.label(num),
                song_dir: format!("{}{num}", song_type.prefix()),
                verses,
            });
            true
        } else {
            false
        }
    }

    /// Add specific verses of a song to the liturgy and rebuild the slide list.
    pub fn add_song_with_verses(&mut self, song_type: SongType, num: u32, verses: Vec<u32>) {
        if verses.is_empty() {
            return;
        }
        record_hymn_usage(song_type, num);
        self.liturgy.push(LiturgyEntry {
            song_name: song_type.label(num),
            song_dir: format!("{}{num}", song_type.prefix()),
            verses,
        });
        self.rebuild_slides();
    }

    /// Rebuild the flat slide list from the current liturgy entries, resolving
    /// each verse to its image files on disk. Clears all caches and attempts
    /// to preserve the current slide position.
    pub fn rebuild_slides(&mut self) {
        let prev = self.slides.get(self.current_slide).cloned();
        self.slides.clear();
        self.texture_cache.clear();
        self.rendering.clear();
        self.render_errors.clear();
        self.current_slide = 0;

        for entry in &self.liturgy {
            for &v in &entry.verses {
                let dir = self.songs_dir.join(&entry.song_dir);
                let mut files: Vec<(PathBuf, u32)> = Vec::new();

                if dir.join("notes.ly").exists() && dir.join(format!("lyrics_{v}.ly")).exists() {
                    let n_parts = crate::render_ly::num_parts_for_verse(&dir, v);
                    let suffixes = ["", "a", "b", "c"];
                    for part in 0..n_parts {
                        let suffix = if n_parts > 1 { suffixes.get(part).unwrap_or(&"") } else { &"" };
                        files.push((dir.join(format!("{v}{suffix}.svg")), part as u32));
                    }
                }

                for (path, part) in files {
                    self.slides.push(Slide {
                        title: entry.song_name.clone(),
                        all_verses: entry.verses.clone(),
                        current_verse: v,
                        part,
                        path,
                        song_dir: dir.clone(),
                    });
                }
            }
        }

        // Restore position after rebuild
        if let Some(prev) = prev {
            if let Some(idx) = self
                .slides
                .iter()
                .position(|s| s.title == prev.title && s.current_verse == prev.current_verse)
            {
                self.current_slide = idx;
            }
        }
    }

    /// Move the current slide index by `delta`, clamping to valid bounds.
    pub fn navigate(&mut self, delta: isize) {
        if self.slides.is_empty() {
            return;
        }
        let new = self.current_slide as isize + delta;
        self.current_slide = new.clamp(0, self.slides.len() as isize - 1) as usize;
    }
}

/// Resolve the base song data directory (`lilypond/`).
///
/// When `--update` mode is active the platform data directory is preferred
/// (`%APPDATA%\bop` on Windows, `~/.local/share/bop` on Linux).
/// Otherwise falls back to the current working directory.
pub fn base_dir() -> PathBuf {
    if data_dir_mode() {
        let data = crate::render_ly::data_dir().join("lilypond");
        if data.exists() {
            return data;
        }
    }
    std::env::current_dir().unwrap_or_default().join("lilypond")
}
