use clap::Parser;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use gtk4 as gtk;
use gtk::gdk;

use crate::rendering::DEFAULT_RENDER_WIDTH;

// ── CLI ─────────────────────────────────────────────────────────────

/// Book of Praise — hymn and psalm display application
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Use PNG images instead of SVG
    #[arg(long)]
    pub png: bool,

    /// Psalm numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    pub psalm: Vec<u32>,

    /// Hymn numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    pub hymn: Vec<u32>,

    /// Check for updates on startup
    #[arg(long, default_value_t = cfg!(feature = "auto-update"))]
    pub update: bool,
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

/// Read the verification count for a specific verse from its `verify_N.txt` file.
pub fn read_verify_count(song_dir: &Path, verse: u32) -> u32 {
    let path = song_dir.join(format!("verify_{verse}.txt"));
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Increment and persist the verification count for a verse, returning the new count.
pub fn increment_verify(song_dir: &Path, verse: u32) -> u32 {
    let count = read_verify_count(song_dir, verse) + 1;
    let path = song_dir.join(format!("verify_{verse}.txt"));
    let _ = std::fs::write(path, count.to_string());
    count
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
    pub path: PathBuf,
    pub song_dir: PathBuf,
}

pub type CacheKey = (PathBuf, u32, u32); // (path, verse, render_width)

pub struct AppState {
    pub songs_dir: PathBuf,
    pub library: SongLibrary,
    pub liturgy: Vec<LiturgyEntry>,
    pub slides: Vec<Slide>,
    pub current_slide: usize,
    pub use_svg: bool,
    pub render_width: u32,
    pub texture_cache: HashMap<CacheKey, gdk::Texture>,
    pub rendering: HashSet<(PathBuf, u32)>,
    pub render_errors: HashMap<(PathBuf, u32), String>,
    pub verified_this_session: HashSet<(PathBuf, u32)>,
    /// Song dirs that had edits saved this session (for email-patch-on-close).
    pub edited_song_dirs: HashSet<PathBuf>,
}

impl AppState {
    /// Initialize application state from CLI arguments. Loads the song library,
    /// adds any songs specified on the command line (defaulting to Psalm 1), and
    /// builds the initial slide list.
    pub fn new(cli: &Cli) -> Self {
        let use_svg = !cli.png;
        let songs_dir = base_dir(use_svg);
        let library = SongLibrary::scan(&songs_dir);

        let mut state = AppState {
            songs_dir,
            library,
            liturgy: Vec::new(),
            slides: Vec::new(),
            current_slide: 0,
            use_svg,
            render_width: DEFAULT_RENDER_WIDTH,
            texture_cache: HashMap::new(),
            rendering: HashSet::new(),
            render_errors: HashMap::new(),
            verified_this_session: HashSet::new(),
            edited_song_dirs: HashSet::new(),
        };

        // Load songs from CLI, defaulting to Psalm 1
        let mut any = false;
        for &n in &cli.psalm {
            any |= state.add_song(SongType::Psalm, n);
        }
        for &n in &cli.hymn {
            any |= state.add_song(SongType::Hymn, n);
        }
        if !any {
            state.add_song(SongType::Psalm, 1);
        }
        state.rebuild_slides();
        state
    }

    /// Add all verses of a song to the liturgy. Returns `true` if the song was found.
    pub fn add_song(&mut self, song_type: SongType, num: u32) -> bool {
        if let Some(verses) = self.library.get(song_type, num).cloned() {
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
                let mut files = if self.use_svg {
                    // In SVG mode, skip pre-existing SVGs so the render pipeline is used
                    Vec::new()
                } else {
                    find_verse_files(&dir, v)
                };

                // In SVG mode, create slide for renderable sources even if SVG missing
                if files.is_empty() && self.use_svg {
                    if dir.join("notes.ly").exists() && dir.join(format!("lyrics_{v}.ly")).exists()
                    {
                        files.push(dir.join(format!("{v}.svg")));
                    }
                }

                for path in files {
                    self.slides.push(Slide {
                        title: entry.song_name.clone(),
                        all_verses: entry.verses.clone(),
                        current_verse: v,
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

    /// Switch between SVG and PNG rendering mode, rescanning the library and
    /// rebuilding slides from the appropriate data directory.
    pub fn set_use_svg(&mut self, use_svg: bool) {
        if self.use_svg == use_svg {
            return;
        }
        self.use_svg = use_svg;
        self.songs_dir = base_dir(use_svg);
        self.library = SongLibrary::scan(&self.songs_dir);
        self.rebuild_slides();
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

/// Return the directory containing the running executable, falling back to the
/// current working directory.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

/// Resolve the base song data directory (`lilypond/` for SVG mode, `photos/` for PNG),
/// checking the current working directory first, then falling back to the executable's directory.
pub fn base_dir(use_svg: bool) -> PathBuf {
    let sub = if use_svg { "lilypond" } else { "photos" };
    let cwd = std::env::current_dir().unwrap_or_default().join(sub);
    if cwd.exists() {
        return cwd;
    }
    exe_dir().join(sub)
}

/// Find image files for a verse number, sorted (handles multi-part: 1a.svg, 1b.svg).
pub fn find_verse_files(dir: &Path, verse: u32) -> Vec<PathBuf> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let ext = Path::new(&name)
            .extension()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if ext != "svg" && ext != "png" {
            continue;
        }
        let stem = Path::new(&name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let num_str: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
        let suffix: String = stem.chars().skip_while(|c| c.is_ascii_digit()).collect();
        let is_valid_suffix =
            suffix.is_empty() || (suffix.len() == 1 && suffix.starts_with(|c: char| c.is_ascii_lowercase()));
        if num_str.parse::<u32>().ok() == Some(verse) && is_valid_suffix {
            files.push((suffix, entry.path()));
        }
    }
    files.sort();
    files.into_iter().map(|(_, p)| p).collect()
}
