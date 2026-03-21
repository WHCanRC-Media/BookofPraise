#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod render_ly;

use clap::Parser;
use gtk4 as gtk;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// ── Update check ────────────────────────────────────────────────────

const GITHUB_REPO: &str = "vanjoe/bookOfPraise";
const GITHUB_PAT: &str = "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";
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
fn check_for_update() -> Result<Option<(String, String)>, String> {
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
fn download_and_extract(asset_url: &str, tag: &str) -> Result<(), String> {
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

// ── CLI ─────────────────────────────────────────────────────────────

/// Book of Praise — hymn and psalm display application
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Use PNG images instead of SVG
    #[arg(long)]
    png: bool,

    /// Psalm numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    psalm: Vec<u32>,

    /// Hymn numbers to load on startup (repeatable)
    #[arg(long, value_name = "NUM")]
    hymn: Vec<u32>,

    /// Check for updates on startup
    #[arg(long, default_value_t = cfg!(feature = "auto-update"))]
    update: bool,
}

// ── Data model ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum SongType {
    Psalm,
    Hymn,
}

impl SongType {
    fn prefix(self) -> &'static str {
        match self {
            SongType::Psalm => "psalm",
            SongType::Hymn => "hymn",
        }
    }

    fn label(self, num: u32) -> String {
        match self {
            SongType::Psalm => format!("Psalm {num}"),
            SongType::Hymn => format!("Hymn {num}"),
        }
    }
}

struct SongLibrary {
    psalms: BTreeMap<u32, Vec<u32>>,
    hymns: BTreeMap<u32, Vec<u32>>,
}

impl SongLibrary {
    fn scan(dir: &Path) -> Self {
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

    fn get(&self, song_type: SongType, num: u32) -> Option<&Vec<u32>> {
        match song_type {
            SongType::Psalm => self.psalms.get(&num),
            SongType::Hymn => self.hymns.get(&num),
        }
    }
}

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

fn read_verify_count(song_dir: &Path, verse: u32) -> u32 {
    let path = song_dir.join(format!("verify_{verse}.txt"));
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn increment_verify(song_dir: &Path, verse: u32) -> u32 {
    let count = read_verify_count(song_dir, verse) + 1;
    let path = song_dir.join(format!("verify_{verse}.txt"));
    let _ = std::fs::write(path, count.to_string());
    count
}

#[derive(Clone)]
struct LiturgyEntry {
    song_name: String,
    song_dir: String,
    verses: Vec<u32>,
}

#[derive(Clone)]
struct Slide {
    title: String,
    all_verses: Vec<u32>,
    current_verse: u32,
    path: PathBuf,
    song_dir: PathBuf,
}

type CacheKey = (PathBuf, u32, u32); // (path, verse, render_width)

struct AppState {
    songs_dir: PathBuf,
    library: SongLibrary,
    liturgy: Vec<LiturgyEntry>,
    slides: Vec<Slide>,
    current_slide: usize,
    use_svg: bool,
    render_width: u32,
    texture_cache: HashMap<CacheKey, gdk::Texture>,
    rendering: HashSet<(PathBuf, u32)>,
    render_errors: HashMap<(PathBuf, u32), String>,
    verified_this_session: HashSet<(PathBuf, u32)>,
}

impl AppState {
    fn new(cli: &Cli) -> Self {
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

    fn add_song(&mut self, song_type: SongType, num: u32) -> bool {
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

    fn add_song_with_verses(&mut self, song_type: SongType, num: u32, verses: Vec<u32>) {
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

    fn rebuild_slides(&mut self) {
        let prev = self.slides.get(self.current_slide).cloned();
        self.slides.clear();
        self.texture_cache.clear();
        self.rendering.clear();
        self.render_errors.clear();
        self.current_slide = 0;

        for entry in &self.liturgy {
            for &v in &entry.verses {
                let dir = self.songs_dir.join(&entry.song_dir);
                let mut files = find_verse_files(&dir, v);

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

    fn set_use_svg(&mut self, use_svg: bool) {
        if self.use_svg == use_svg {
            return;
        }
        self.use_svg = use_svg;
        self.songs_dir = base_dir(use_svg);
        self.library = SongLibrary::scan(&self.songs_dir);
        self.rebuild_slides();
    }

    fn navigate(&mut self, delta: isize) {
        if self.slides.is_empty() {
            return;
        }
        let new = self.current_slide as isize + delta;
        self.current_slide = new.clamp(0, self.slides.len() as isize - 1) as usize;
    }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

fn base_dir(use_svg: bool) -> PathBuf {
    let sub = if use_svg { "lilypond" } else { "photos" };
    let cwd = std::env::current_dir().unwrap_or_default().join(sub);
    if cwd.exists() {
        return cwd;
    }
    exe_dir().join(sub)
}

/// Find image files for a verse number, sorted (handles multi-part: 1a.svg, 1b.svg).
fn find_verse_files(dir: &Path, verse: u32) -> Vec<PathBuf> {
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

// ── Image rendering ─────────────────────────────────────────────────

const DEFAULT_RENDER_WIDTH: u32 = 2400;

type Pixmap = resvg::tiny_skia::Pixmap;

fn load_svg_pixmap(path: &Path, render_width: u32) -> Option<Pixmap> {
    let data = std::fs::read(path).ok()?;
    let data = String::from_utf8_lossy(&data)
        .replace("currentColor", "black")
        .into_bytes();
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(&data, &opt).ok()?;
    let size = tree.size();
    if size.width() == 0.0 || size.height() == 0.0 {
        return None;
    }
    let scale = render_width as f32 / size.width();
    let w = (size.width() * scale) as u32;
    let h = (size.height() * scale) as u32;
    let mut pm = Pixmap::new(w, h)?;
    pm.fill(resvg::tiny_skia::Color::WHITE);
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pm.as_mut(),
    );
    Some(pm)
}

fn load_png_pixmap(path: &Path, render_width: u32) -> Option<Pixmap> {
    let img = image::open(path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    let scale = render_width as f32 / w as f32;
    let th = (h as f32 * scale) as u32;
    let scaled =
        image::imageops::resize(&img, render_width, th, image::imageops::FilterType::Lanczos3);

    let mut pm = Pixmap::new(render_width, th)?;
    pm.fill(resvg::tiny_skia::Color::WHITE);
    for (i, px) in scaled.pixels().enumerate() {
        if let Some(c) =
            resvg::tiny_skia::PremultipliedColorU8::from_rgba(px[0], px[1], px[2], px[3])
        {
            pm.pixels_mut()[i] = c;
        }
    }
    Some(pm)
}

/// Crop white edges, center in 16:9 frame with title padding.
fn crop_and_frame(src: &Pixmap, render_width: u32) -> Option<Pixmap> {
    let output_w = render_width;
    let output_h = render_width * 9 / 16;
    let title_pad = render_width / 30;

    let (w, h) = (src.width() as usize, src.height() as usize);
    let px = src.pixels();
    let white =
        |p: resvg::tiny_skia::PremultipliedColorU8| p.red() > 250 && p.green() > 250 && p.blue() > 250;

    // Find content bounds
    let top = (0..h).find(|&y| (0..w).any(|x| !white(px[y * w + x]))).unwrap_or(0);
    let bot = (0..h).rfind(|&y| (0..w).any(|x| !white(px[y * w + x]))).unwrap_or(h - 1);
    let left = (0..w).find(|&x| (top..=bot).any(|y| !white(px[y * w + x]))).unwrap_or(0);
    let right = (0..w).rfind(|&x| (top..=bot).any(|y| !white(px[y * w + x]))).unwrap_or(w - 1);

    let margin = 4;
    let top = top.saturating_sub(margin);
    let left = left.saturating_sub(margin);
    let bot = (bot + margin).min(h - 1);
    let right = (right + margin).min(w - 1);
    let (cw, ch) = (right - left + 1, bot - top + 1);

    // Scale content to fit frame
    let avail_h = (output_h - title_pad) as usize;
    let scale = (output_w as f32 / cw as f32)
        .min(avail_h as f32 / ch as f32);
    let (sw, sh) = ((cw as f32 * scale) as usize, (ch as f32 * scale) as usize);
    let x_off = (output_w as usize - sw) / 2;
    let y_off = title_pad as usize + (avail_h - sh) / 2;

    let mut out = Pixmap::new(output_w, output_h)?;
    out.fill(resvg::tiny_skia::Color::WHITE);
    for dy in 0..sh {
        for dx in 0..sw {
            let sx = ((dx as f32 / scale) as usize).min(cw - 1);
            let sy = ((dy as f32 / scale) as usize).min(ch - 1);
            out.pixels_mut()[(y_off + dy) * output_w as usize + x_off + dx] =
                px[(top + sy) * w + left + sx];
        }
    }
    Some(out)
}

fn render_title(pixmap: &mut Pixmap, slide: &Slide, render_width: u32) {
    let font_size = render_width as f32 / 40.0;

    let verses: String = slide
        .all_verses
        .iter()
        .map(|&v| {
            let color = if v == slide.current_verse { "black" } else { "grey" };
            format!(r#"<tspan fill="{color}">{v} </tspan>"#)
        })
        .collect();

    let title = slide
        .title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let w = pixmap.width();
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
          <text x="{cx}" y="{y}" font-family="serif" font-size="{fs}"
                text-anchor="middle" fill="black">
            <tspan>{title}: </tspan>{verses}
          </text>
        </svg>"#,
        h = (font_size * 2.0) as u32,
        cx = w / 2,
        y = font_size * 1.3,
        fs = font_size,
    );

    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    if let Ok(tree) = resvg::usvg::Tree::from_data(svg.as_bytes(), &opt) {
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );
    }
}

fn load_slide_texture(slide: &Slide, render_width: u32) -> Option<gdk::Texture> {
    let is_svg = slide
        .path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));

    let raw = if is_svg {
        load_svg_pixmap(&slide.path, render_width)?
    } else {
        load_png_pixmap(&slide.path, render_width)?
    };

    let mut framed = crop_and_frame(&raw, render_width)?;
    render_title(&mut framed, slide, render_width);

    let (w, h) = (framed.width(), framed.height());
    let bytes = glib::Bytes::from(framed.data());
    Some(
        gdk::MemoryTexture::new(w as i32, h as i32, gdk::MemoryFormat::R8g8b8a8, &bytes, (w * 4) as usize)
            .upcast(),
    )
}

// ── UI helpers ──────────────────────────────────────────────────────

fn load_editor_contents(state: &AppState, notes_view: &gtk::TextView, lyrics_view: &gtk::TextView, lyrics_label: &gtk::Label) {
    if let Some(slide) = state.slides.get(state.current_slide) {
        let notes_path = slide.song_dir.join("notes.ly");
        let lyrics_path = slide.song_dir.join(format!("lyrics_{}.ly", slide.current_verse));

        let notes_text = std::fs::read_to_string(&notes_path).unwrap_or_default();
        notes_view.buffer().set_text(&notes_text);

        let lyrics_text = std::fs::read_to_string(&lyrics_path).unwrap_or_default();
        lyrics_view.buffer().set_text(&lyrics_text);

        lyrics_label.set_text(&format!("lyrics_{}.ly", slide.current_verse));
    } else {
        notes_view.buffer().set_text("");
        lyrics_view.buffer().set_text("");
        lyrics_label.set_text("lyrics.ly");
    }
}

fn save_editor_contents(state: &AppState, notes_view: &gtk::TextView, lyrics_view: &gtk::TextView) {
    if let Some(slide) = state.slides.get(state.current_slide) {
        let notes_path = slide.song_dir.join("notes.ly");
        let lyrics_path = slide.song_dir.join(format!("lyrics_{}.ly", slide.current_verse));

        let buf = notes_view.buffer();
        let notes_text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let _ = std::fs::write(&notes_path, notes_text.as_str());

        let buf = lyrics_view.buffer();
        let lyrics_text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let _ = std::fs::write(&lyrics_path, lyrics_text.as_str());
    }
}

fn needs_render(slide: &Slide) -> bool {
    let is_svg = slide
        .path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    is_svg && !render_ly::is_svg_current(&slide.song_dir, slide.current_verse)
}

fn refresh_display(
    state_rc: &Rc<RefCell<AppState>>,
    picture: &gtk::Picture,
    nav_label: &gtk::Label,
    spinner: &gtk::Spinner,
    error_label: &gtk::Label,
    verify_btn: &gtk::Button,
) {
    let mut state = state_rc.borrow_mut();
    spinner.stop();
    spinner.set_visible(false);
    error_label.set_visible(false);

    // Render at the picture's actual pixel width
    let w = picture.width();
    let scale = picture.scale_factor();
    if w > 0 {
        let pixel_width = (w as u32) * (scale as u32);
        state.render_width = pixel_width.max(DEFAULT_RENDER_WIDTH);
    }
    let render_width = state.render_width;
    let slide_info = state.slides.get(state.current_slide).map(|slide| {
        (
            (slide.path.clone(), slide.current_verse, render_width),
            (slide.path.clone(), slide.current_verse),
            slide.song_dir.clone(),
            slide.current_verse,
            state.current_slide,
            state.slides.len(),
        )
    });

    if let Some((cache_key, render_key, song_dir, verse, idx, total)) = slide_info {
        nav_label.set_text(&format!("{}/{}", idx + 1, total));

        // Update verify button state
        let verify_count = read_verify_count(&song_dir, verse);
        let verified_session = state.verified_this_session.contains(&(song_dir.clone(), verse));
        if verify_count >= 2 || verified_session {
            verify_btn.set_label("Verified");
            verify_btn.set_sensitive(false);
        } else {
            verify_btn.set_label("Verify");
            verify_btn.set_sensitive(true);
        }

        // Check for render error
        if let Some(err) = state.render_errors.get(&render_key) {
            picture.set_paintable(None::<&gdk::Texture>);
            error_label.set_text(err);
            error_label.set_visible(true);
            return;
        }

        // Try loading from cache or disk
        let tex = state.texture_cache.get(&cache_key).cloned().or_else(|| {
            load_slide_texture(&state.slides[idx], render_width)
        });
        if let Some(tex) = tex {
            picture.set_paintable(Some(&tex));
            state.texture_cache.insert(cache_key, tex);
            return;
        }

        // Need to render — show spinner and spawn background thread
        let should_render = !state.rendering.contains(&render_key) && needs_render(&state.slides[idx]);
        if should_render {
            state.rendering.insert(render_key.clone());

            // Must drop the borrow before setting up the callback
            drop(state);

            spinner.set_visible(true);
            spinner.start();
            picture.set_paintable(None::<&gdk::Texture>);

            let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
            std::thread::spawn(move || {
                let result = render_ly::render_svg(&song_dir, verse);
                let _ = tx.send(result);
            });

            // Poll for completion from the main thread
            let state_rc2 = state_rc.clone();
            let picture2 = picture.clone();
            let nav_label2 = nav_label.clone();
            let spinner2 = spinner.clone();
            let error_label2 = error_label.clone();
            let verify_btn2 = verify_btn.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                match rx.try_recv() {
                    Ok(result) => {
                        {
                            let mut state = state_rc2.borrow_mut();
                            state.rendering.remove(&render_key);
                            if let Err(err) = result {
                                state.render_errors.insert(render_key.clone(), err);
                            }
                        }
                        refresh_display(&state_rc2, &picture2, &nav_label2, &spinner2, &error_label2, &verify_btn2);
                        glib::ControlFlow::Break
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(_) => glib::ControlFlow::Break, // sender dropped
                }
            });
            return;
        }

        // Rendering in progress
        if state.rendering.contains(&render_key) {
            spinner.set_visible(true);
            spinner.start();
            picture.set_paintable(None::<&gdk::Texture>);
        }
    } else {
        picture.set_paintable(None::<&gdk::Texture>);
        nav_label.set_text("0/0");
        verify_btn.set_label("Verify");
        verify_btn.set_sensitive(false);
    }
}

fn refresh_liturgy(state: &AppState, label: &gtk::Label) {
    if state.liturgy.is_empty() {
        label.set_text("");
    } else {
        let parts: Vec<String> = state
            .liturgy
            .iter()
            .map(|e| {
                let vs = e.verses.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
                format!("{} ({})", e.song_name, vs)
            })
            .collect();
        label.set_text(&format!("Liturgy: {}", parts.join(" | ")));
    }
}

fn rebuild_verse_checks(verse_box: &gtk::FlowBox, verses: &[u32]) {
    while let Some(child) = verse_box.first_child() {
        verse_box.remove(&child);
    }
    for &v in verses {
        let check = gtk::CheckButton::with_label(&format!("V{v}"));
        check.set_widget_name(&v.to_string());
        verse_box.insert(&check, -1);
    }
}

fn checked_verses(verse_box: &gtk::FlowBox) -> Vec<u32> {
    let mut out = Vec::new();
    let mut child = verse_box.first_child();
    while let Some(w) = child {
        if let Some(fb) = w.downcast_ref::<gtk::FlowBoxChild>() {
            if let Some(cb) = fb.child().and_then(|c| c.downcast::<gtk::CheckButton>().ok()) {
                if cb.is_active() {
                    if let Ok(v) = cb.widget_name().parse::<u32>() {
                        out.push(v);
                    }
                }
            }
        }
        child = w.next_sibling();
    }
    out.sort();
    out
}

fn check_all(verse_box: &gtk::FlowBox) {
    let mut child = verse_box.first_child();
    while let Some(w) = child {
        if let Some(fb) = w.downcast_ref::<gtk::FlowBoxChild>() {
            if let Some(cb) = fb.child().and_then(|c| c.downcast::<gtk::CheckButton>().ok()) {
                cb.set_active(true);
            }
        }
        child = w.next_sibling();
    }
}

// ── UI construction ─────────────────────────────────────────────────

fn build_ui(app: &gtk::Application, cli: &Cli) {
    let state = Rc::new(RefCell::new(AppState::new(cli)));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Book of Praise")
        .default_width(1024)
        .default_height(768)
        .build();
    // Image display
    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_vexpand(true);
    picture.set_hexpand(true);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.add_css_class("slide-image");

    // Loading spinner (centered over picture area)
    let spinner = gtk::Spinner::new();
    spinner.set_width_request(48);
    spinner.set_height_request(48);
    spinner.set_halign(gtk::Align::Center);
    spinner.set_valign(gtk::Align::Center);
    spinner.set_visible(false);

    // Error label (centered over picture area, wrapping)
    let error_label = gtk::Label::new(None);
    error_label.set_wrap(true);
    error_label.set_halign(gtk::Align::Center);
    error_label.set_valign(gtk::Align::Center);
    error_label.set_visible(false);
    error_label.set_selectable(true);
    error_label.set_margin_start(24);
    error_label.set_margin_end(24);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&picture));
    overlay.add_overlay(&spinner);
    overlay.add_overlay(&error_label);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&overlay));
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);

    // Editor panel (right side, hidden by default)
    let notes_view = gtk::TextView::new();
    notes_view.set_monospace(true);
    notes_view.set_wrap_mode(gtk::WrapMode::Word);
    notes_view.add_css_class("editor-text");

    let notes_scroll = gtk::ScrolledWindow::new();
    notes_scroll.set_child(Some(&notes_view));
    notes_scroll.set_vexpand(true);

    let lyrics_view = gtk::TextView::new();
    lyrics_view.set_monospace(true);
    lyrics_view.set_wrap_mode(gtk::WrapMode::Word);
    lyrics_view.add_css_class("editor-text");

    let lyrics_scroll = gtk::ScrolledWindow::new();
    lyrics_scroll.set_child(Some(&lyrics_view));
    lyrics_scroll.set_vexpand(true);

    let notes_label = gtk::Label::new(Some("notes.ly"));
    notes_label.set_xalign(0.0);
    let lyrics_label = gtk::Label::new(Some("lyrics.ly"));
    lyrics_label.set_xalign(0.0);

    let save_btn = gtk::Button::with_label("Save & Re-render");

    let editor_panel = gtk::Box::new(gtk::Orientation::Vertical, 4);
    editor_panel.set_margin_start(4);
    editor_panel.set_margin_end(4);
    editor_panel.set_margin_top(4);
    editor_panel.set_margin_bottom(4);
    editor_panel.append(&notes_label);
    editor_panel.append(&notes_scroll);
    editor_panel.append(&lyrics_label);
    editor_panel.append(&lyrics_scroll);
    editor_panel.append(&save_btn);
    editor_panel.set_visible(false);
    editor_panel.set_hexpand(true);

    // Horizontal paned: image left, editor right
    let hpaned = gtk::Paned::new(gtk::Orientation::Horizontal);
    hpaned.set_start_child(Some(&scroll));
    hpaned.set_end_child(Some(&editor_panel));
    hpaned.set_resize_start_child(true);
    hpaned.set_resize_end_child(true);
    hpaned.set_vexpand(true);

    // Liturgy label
    let liturgy_label = gtk::Label::new(None);
    liturgy_label.set_xalign(0.0);
    liturgy_label.set_wrap(true);

    // Navigation row
    let prev_btn = gtk::Button::with_label("Prev");
    let nav_label = gtk::Label::new(Some("0/0"));
    let next_btn = gtk::Button::with_label("Next");
    let clear_btn = gtk::Button::with_label("Clear liturgy");
    let edit_btn = gtk::ToggleButton::with_label("Edit");
    edit_btn.set_visible(!cli.png); // only show in SVG mode
    let verify_btn = gtk::Button::with_label("Verify");
    verify_btn.set_visible(!cli.png); // only show in SVG mode
    let png_label = gtk::Label::new(Some("PNG"));
    let svg_switch = gtk::Switch::new();
    svg_switch.set_active(!cli.png);
    let svg_label = gtk::Label::new(Some("SVG"));

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let nav_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    for w in [
        prev_btn.upcast_ref::<gtk::Widget>(),
        nav_label.upcast_ref(),
        next_btn.upcast_ref(),
        clear_btn.upcast_ref(),
        spacer.upcast_ref(),
        edit_btn.upcast_ref(),
        verify_btn.upcast_ref(),
        png_label.upcast_ref(),
        svg_switch.upcast_ref(),
        svg_label.upcast_ref(),
    ] {
        nav_row.append(w);
    }

    // Song input row
    let psalm_radio = gtk::CheckButton::with_label("Psalm");
    let hymn_radio = gtk::CheckButton::with_label("Hymn");
    hymn_radio.set_group(Some(&psalm_radio));
    psalm_radio.set_active(true);

    let number_entry = gtk::Entry::new();
    number_entry.set_width_chars(5);

    let verse_box = gtk::FlowBox::new();
    verse_box.set_selection_mode(gtk::SelectionMode::None);
    verse_box.set_max_children_per_line(20);
    verse_box.set_hexpand(true);

    let all_btn = gtk::Button::with_label("All");
    let add_btn = gtk::Button::with_label("Add");

    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    input_row.set_valign(gtk::Align::Center);
    for w in [
        psalm_radio.upcast_ref::<gtk::Widget>(),
        hymn_radio.upcast_ref(),
        gtk::Label::new(Some("#")).upcast_ref(),
        number_entry.upcast_ref(),
        verse_box.upcast_ref(),
        all_btn.upcast_ref(),
        add_btn.upcast_ref(),
    ] {
        input_row.append(w);
    }

    // Controls container
    let controls = gtk::Box::new(gtk::Orientation::Vertical, 4);
    controls.set_margin_start(8);
    controls.set_margin_end(8);
    controls.set_margin_top(4);
    controls.set_margin_bottom(8);
    controls.append(&liturgy_label);
    controls.append(&nav_row);
    controls.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    controls.append(&input_row);

    // Main layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.append(&hpaned);
    vbox.append(&controls);
    window.set_child(Some(&vbox));

    // Initial display
    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
    refresh_liturgy(&state.borrow(), &liturgy_label);

    // ── Helpers for signal closures ──

    // Macro to reduce clone boilerplate in signal handlers
    macro_rules! connect {
        ($widget:expr, $method:ident, [$($clone:ident),*], $body:expr) => {{
            $(let $clone = $clone.clone();)*
            $widget.$method($body);
        }};
    }

    // Song type from radio state
    let song_type = {
        let psalm_radio = psalm_radio.clone();
        move || {
            if psalm_radio.is_active() {
                SongType::Psalm
            } else {
                SongType::Hymn
            }
        }
    };

    // ── Signal connections ──

    // Number entry → update verse checkboxes
    connect!(number_entry, connect_changed, [state, verse_box, song_type], move |entry| {
        if let Ok(num) = entry.text().parse::<u32>() {
            let verses = state.borrow().library.get(song_type(), num).cloned().unwrap_or_default();
            rebuild_verse_checks(&verse_box, &verses);
        } else {
            rebuild_verse_checks(&verse_box, &[]);
        }
    });

    // Radio toggle → update verse checkboxes
    connect!(psalm_radio, connect_active_notify, [state, verse_box, number_entry, song_type], move |_| {
        if let Ok(num) = number_entry.text().parse::<u32>() {
            let verses = state.borrow().library.get(song_type(), num).cloned().unwrap_or_default();
            rebuild_verse_checks(&verse_box, &verses);
        }
    });

    // Prev / Next
    connect!(prev_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, editor_panel, notes_view, lyrics_view, lyrics_label], move |_| {
        state.borrow_mut().navigate(-1);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
        if editor_panel.is_visible() {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &lyrics_label);
        }
    });
    connect!(next_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, editor_panel, notes_view, lyrics_view, lyrics_label], move |_| {
        state.borrow_mut().navigate(1);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
        if editor_panel.is_visible() {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &lyrics_label);
        }
    });

    // Clear
    connect!(clear_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, liturgy_label], move |_| {
        { let mut s = state.borrow_mut(); s.liturgy.clear(); s.rebuild_slides(); }
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
        refresh_liturgy(&state.borrow(), &liturgy_label);
    });

    // Edit toggle
    connect!(edit_btn, connect_toggled, [state, editor_panel, notes_view, lyrics_view, lyrics_label], move |btn| {
        let active = btn.is_active();
        editor_panel.set_visible(active);
        if active {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &lyrics_label);
        }
    });

    // Verify button
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let spinner = spinner.clone();
        let error_label = error_label.clone();
        let verify_btn2 = verify_btn.clone();
        verify_btn.connect_clicked(move |_| {
            {
                let mut s = state.borrow_mut();
                if let Some(slide) = s.slides.get(s.current_slide) {
                    let song_dir = slide.song_dir.clone();
                    let verse = slide.current_verse;
                    increment_verify(&song_dir, verse);
                    s.verified_this_session.insert((song_dir, verse));
                }
            }
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn2);
        });
    }

    // Save & Re-render
    connect!(save_btn, connect_clicked, [state, notes_view, lyrics_view, picture, nav_label, spinner, error_label, verify_btn], move |_| {
        save_editor_contents(&state.borrow(), &notes_view, &lyrics_view);
        {
            let mut s = state.borrow_mut();
            // Remove cached texture and errors so it re-renders
            if let Some(slide) = s.slides.get(s.current_slide) {
                let path = slide.path.clone();
                let verse = slide.current_verse;
                s.texture_cache.retain(|k, _| k.0 != path || k.1 != verse);
                s.render_errors.remove(&(path.clone(), verse));
                // Delete the SVG so lilypond re-renders it
                let _ = std::fs::remove_file(&path);
            }
        }
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
    });

    // SVG/PNG toggle
    connect!(svg_switch, connect_active_notify, [state, picture, nav_label, spinner, error_label, verify_btn, liturgy_label, number_entry, verse_box, edit_btn, editor_panel], move |sw| {
        state.borrow_mut().set_use_svg(sw.is_active());
        edit_btn.set_active(false);
        edit_btn.set_visible(sw.is_active());
        verify_btn.set_visible(sw.is_active());
        editor_panel.set_visible(false);
        number_entry.set_text("");
        rebuild_verse_checks(&verse_box, &[]);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
        refresh_liturgy(&state.borrow(), &liturgy_label);
    });

    // All button — check all and add immediately
    connect!(all_btn, connect_clicked, [state, verse_box, number_entry, song_type, picture, nav_label, spinner, error_label, verify_btn, liturgy_label], move |_| {
        check_all(&verse_box);
        if let Ok(num) = number_entry.text().parse::<u32>() {
            let verses = checked_verses(&verse_box);
            state.borrow_mut().add_song_with_verses(song_type(), num, verses);
            number_entry.set_text("");
            rebuild_verse_checks(&verse_box, &[]);
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
            refresh_liturgy(&state.borrow(), &liturgy_label);
        }
    });

    // Add button
    connect!(add_btn, connect_clicked, [state, verse_box, number_entry, song_type, picture, nav_label, spinner, error_label, verify_btn, liturgy_label], move |_| {
        if let Ok(num) = number_entry.text().parse::<u32>() {
            let verses = checked_verses(&verse_box);
            state.borrow_mut().add_song_with_verses(song_type(), num, verses);
            number_entry.set_text("");
            rebuild_verse_checks(&verse_box, &[]);
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
            refresh_liturgy(&state.borrow(), &liturgy_label);
        }
    });

    // Arrow key navigation
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let spinner = spinner.clone();
        let error_label = error_label.clone();
        let verify_btn = verify_btn.clone();
        let editor_panel = editor_panel.clone();
        let notes_view = notes_view.clone();
        let lyrics_view = lyrics_view.clone();
        let lyrics_label = lyrics_label.clone();
        let kc = gtk::EventControllerKey::new();
        kc.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk::Key::s && modifiers.contains(gdk::ModifierType::CONTROL_MASK) && editor_panel.is_visible() {
                save_editor_contents(&state.borrow(), &notes_view, &lyrics_view);
                {
                    let mut s = state.borrow_mut();
                    if let Some(slide) = s.slides.get(s.current_slide) {
                        let path = slide.path.clone();
                        let verse = slide.current_verse;
                        s.texture_cache.retain(|k, _| k.0 != path || k.1 != verse);
                        s.render_errors.remove(&(path.clone(), verse));
                        let _ = std::fs::remove_file(&path);
                    }
                }
                refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
                return glib::Propagation::Stop;
            }
            match key {
                gdk::Key::Left => {
                    state.borrow_mut().navigate(-1);
                    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
                    if editor_panel.is_visible() {
                        load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &lyrics_label);
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Right => {
                    state.borrow_mut().navigate(1);
                    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn);
                    if editor_panel.is_visible() {
                        load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &lyrics_label);
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        window.add_controller(kc);
    }

    window.present();

    // ── Startup update check ──
    if cli.update {
        let window = window.clone();
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let spinner = spinner.clone();
        let error_label = error_label.clone();
        let verify_btn = verify_btn.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(check_for_update());
        });
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            match rx.try_recv() {
                Ok(Ok(Some((tag, asset_url)))) => {
                    let dialog = gtk::MessageDialog::new(
                        Some(&window),
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Question,
                        gtk::ButtonsType::YesNo,
                        &format!("Lilypond update available: {tag}\nDownload and install?"),
                    );
                    let window2 = window.clone();
                    let state2 = state.clone();
                    let picture2 = picture.clone();
                    let nav_label2 = nav_label.clone();
                    let spinner2 = spinner.clone();
                    let error_label2 = error_label.clone();
                    let verify_btn2 = verify_btn.clone();
                    let asset_url = Rc::new(asset_url);
                    dialog.connect_response(move |dlg, resp| {
                        dlg.close();
                        if resp == gtk::ResponseType::Yes {
                            let asset_url = (*asset_url).clone();
                            // Show a progress dialog while downloading
                            let progress = gtk::MessageDialog::new(
                                Some(&window2),
                                gtk::DialogFlags::MODAL,
                                gtk::MessageType::Info,
                                gtk::ButtonsType::None,
                                "Downloading update...",
                            );
                            progress.show();

                            let tag2 = tag.clone();
                            let (tx2, rx2) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let _ = tx2.send(download_and_extract(&asset_url, &tag2));
                            });

                            let state3 = state2.clone();
                            let picture3 = picture2.clone();
                            let nav_label3 = nav_label2.clone();
                            let spinner3 = spinner2.clone();
                            let error_label3 = error_label2.clone();
                            let verify_btn3 = verify_btn2.clone();
                            glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                                match rx2.try_recv() {
                                    Ok(Ok(())) => {
                                        progress.close();
                                        // Reload library with new data
                                        {
                                            let mut s = state3.borrow_mut();
                                            s.library = SongLibrary::scan(&s.songs_dir);
                                            s.texture_cache.clear();
                                            s.render_errors.clear();
                                            s.rebuild_slides();
                                        }
                                        refresh_display(&state3, &picture3, &nav_label3, &spinner3, &error_label3, &verify_btn3);
                                        glib::ControlFlow::Break
                                    }
                                    Ok(Err(e)) => {
                                        progress.close();
                                        eprintln!("Update failed: {e}");
                                        glib::ControlFlow::Break
                                    }
                                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                                    Err(_) => { progress.close(); glib::ControlFlow::Break }
                                }
                            });
                        }
                    });
                    dialog.show();
                    glib::ControlFlow::Break
                }
                Ok(_) => glib::ControlFlow::Break, // no update or error
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => glib::ControlFlow::Break,
            }
        });
    }
}

// ── Entry point ─────────────────────────────────────────────────────

fn main() -> glib::ExitCode {
    // Disable client-side decorations so the native Windows titlebar is used.
    #[cfg(target_os = "windows")]
    // SAFETY: called before any other threads are spawned.
    unsafe { std::env::set_var("GTK_CSD", "0") };

    let cli = Cli::parse();
    let app = gtk::Application::builder()
        .application_id("org.bop.bookofpraise")
        .build();
    #[cfg(target_os = "windows")]
    app.connect_startup(|_| {
        let display = gdk::Display::default().expect("Could not connect to a display");
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("win10.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
    let cli = Rc::new(cli);
    app.connect_activate(move |app| build_ui(app, &cli));
    app.run_with_args::<&str>(&[])
}
