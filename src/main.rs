use clap::Parser;
use gtk4 as gtk;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Book of Praise — hymn and psalm display application
#[derive(Parser)]
#[command(version, about)]
struct CliArgs {
    /// Use PNG images instead of SVG
    #[arg(long)]
    png: bool,

    /// Psalm numbers to load on startup (can be repeated)
    #[arg(long, value_name = "NUM")]
    psalm: Vec<u32>,

    /// Hymn numbers to load on startup (can be repeated)
    #[arg(long, value_name = "NUM")]
    hymn: Vec<u32>,
}

fn main() -> glib::ExitCode {
    let cli = CliArgs::parse();

    let app = gtk::Application::builder()
        .application_id("org.bop.bookofpraise")
        .build();

    // Pass CLI args into the activate handler
    let cli = Rc::new(cli);
    app.connect_activate(move |app| build_ui(app, &cli));

    // Pass empty args to GTK so it doesn't try to parse ours
    app.run_with_args::<&str>(&[])
}

// --- Data model (unchanged from egui version) ---

struct SongLibrary {
    psalms: BTreeMap<u32, Vec<u32>>,
    hymns: BTreeMap<u32, Vec<u32>>,
}

impl SongLibrary {
    fn scan(dir: &Path) -> Self {
        let mut psalms = BTreeMap::new();
        let mut hymns = BTreeMap::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let (map, prefix) = if name.starts_with("psalm") {
                    (&mut psalms, "psalm")
                } else if name.starts_with("hymn") {
                    (&mut hymns, "hymn")
                } else {
                    continue;
                };
                if let Ok(num) = name[prefix.len()..].parse::<u32>() {
                    let verses = scan_verses(&entry.path());
                    if !verses.is_empty() {
                        map.insert(num, verses);
                    }
                }
            }
        }
        SongLibrary { psalms, hymns }
    }
}

fn scan_verses(dir: &Path) -> Vec<u32> {
    let mut verses = std::collections::BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let stem = Path::new(&name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let num_part: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = num_part.parse::<u32>() {
                verses.insert(v);
            }
        }
    }
    verses.into_iter().collect()
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
}

#[derive(Clone, Copy, PartialEq)]
enum SongType {
    Psalm,
    Hymn,
}

struct AppState {
    songs_dir: PathBuf,
    library: SongLibrary,
    liturgy: Vec<LiturgyEntry>,
    slides: Vec<Slide>,
    current_slide: usize,
    song_type: SongType,
    available_verses: Vec<u32>,
    use_svg: bool,
}

impl AppState {
    fn new(cli: &CliArgs) -> Self {
        let use_svg = !cli.png;
        let songs_dir = std::env::current_dir()
            .unwrap_or_default()
            .join(if use_svg { "lilypond" } else { "photos" });
        let library = SongLibrary::scan(&songs_dir);

        let mut state = AppState {
            songs_dir,
            library,
            liturgy: Vec::new(),
            slides: Vec::new(),
            current_slide: 0,
            song_type: SongType::Psalm,
            available_verses: Vec::new(),
            use_svg,
        };

        let mut has_songs = false;
        for &num in &cli.psalm {
            if let Some(verses) = state.library.psalms.get(&num).cloned() {
                state.liturgy.push(LiturgyEntry {
                    song_name: format!("Psalm {num}"),
                    song_dir: format!("psalm{num}"),
                    verses,
                });
                has_songs = true;
            }
        }
        for &num in &cli.hymn {
            if let Some(verses) = state.library.hymns.get(&num).cloned() {
                state.liturgy.push(LiturgyEntry {
                    song_name: format!("Hymn {num}"),
                    song_dir: format!("hymn{num}"),
                    verses,
                });
                has_songs = true;
            }
        }
        if !has_songs {
            if let Some(verses) = state.library.psalms.get(&1).cloned() {
                state.liturgy.push(LiturgyEntry {
                    song_name: "Psalm 1".into(),
                    song_dir: "psalm1".into(),
                    verses,
                });
            }
        }
        state.rebuild_slides();

        state
    }

    fn get_available_verses(&self, song_type: SongType, num: u32) -> Vec<u32> {
        let map = match song_type {
            SongType::Psalm => &self.library.psalms,
            SongType::Hymn => &self.library.hymns,
        };
        map.get(&num).cloned().unwrap_or_default()
    }

    fn add_song(&mut self, song_type: SongType, num: u32, verses: Vec<u32>) {
        if verses.is_empty() {
            return;
        }
        let (song_name, prefix) = match song_type {
            SongType::Psalm => (format!("Psalm {num}"), "psalm"),
            SongType::Hymn => (format!("Hymn {num}"), "hymn"),
        };
        self.liturgy.push(LiturgyEntry {
            song_name,
            song_dir: format!("{prefix}{num}"),
            verses,
        });
        self.rebuild_slides();
    }

    fn rebuild_slides(&mut self) {
        self.slides.clear();
        self.current_slide = 0;

        for entry in &self.liturgy {
            for &v in &entry.verses {
                let dir = self.songs_dir.join(&entry.song_dir);
                let mut files = Vec::new();

                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for f in entries.flatten() {
                        let fname = f.file_name().to_string_lossy().to_string();
                        let stem = Path::new(&fname)
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let num_part: String =
                            stem.chars().take_while(|c| c.is_ascii_digit()).collect();
                        let suffix: String =
                            stem.chars().skip_while(|c| c.is_ascii_digit()).collect();
                        if num_part.parse::<u32>().ok() == Some(v)
                            && (suffix.is_empty()
                                || (suffix.len() == 1
                                    && suffix.chars().next().unwrap().is_ascii_lowercase()))
                        {
                            files.push((suffix, f.path()));
                        }
                    }
                }
                files.sort();

                for (_suffix, path) in files {
                    self.slides.push(Slide {
                        title: entry.song_name.clone(),
                        all_verses: entry.verses.clone(),
                        current_verse: v,
                        path,
                    });
                }
            }
        }
    }

    fn set_use_svg(&mut self, use_svg: bool) {
        if self.use_svg == use_svg {
            return;
        }
        // Remember current position
        let prev_slide = self.slides.get(self.current_slide).cloned();

        self.use_svg = use_svg;
        self.songs_dir = std::env::current_dir()
            .unwrap_or_default()
            .join(if use_svg { "lilypond" } else { "photos" });
        self.library = SongLibrary::scan(&self.songs_dir);
        self.rebuild_slides();

        // Restore position by matching title + verse_label
        if let Some(prev) = prev_slide {
            if let Some(idx) = self.slides.iter().position(|s| {
                s.title == prev.title && s.current_verse == prev.current_verse
            }) {
                self.current_slide = idx;
            }
        }
    }

    fn navigate(&mut self, delta: isize) {
        if self.slides.is_empty() {
            return;
        }
        let new = self.current_slide as isize + delta;
        self.current_slide = new.clamp(0, self.slides.len() as isize - 1) as usize;
    }
}

/// Extra pixels at top for the title
const TITLE_PAD: u32 = 80;

/// Crop white edges from a pixmap and add TITLE_PAD at the top.
fn crop_pixmap(src: &resvg::tiny_skia::Pixmap) -> Option<resvg::tiny_skia::Pixmap> {
    let w = src.width() as usize;
    let h = src.height() as usize;
    let pixels = src.pixels();

    let is_white = |px: resvg::tiny_skia::PremultipliedColorU8| {
        px.red() > 250 && px.green() > 250 && px.blue() > 250
    };

    let mut top = 0;
    'top: for y in 0..h {
        for x in 0..w {
            if !is_white(pixels[y * w + x]) { top = y; break 'top; }
        }
    }
    let mut bot = h.saturating_sub(1);
    'bot: for y in (0..h).rev() {
        for x in 0..w {
            if !is_white(pixels[y * w + x]) { bot = y; break 'bot; }
        }
    }
    let mut left = 0;
    'left: for x in 0..w {
        for y in top..=bot {
            if !is_white(pixels[y * w + x]) { left = x; break 'left; }
        }
    }
    let mut right = w.saturating_sub(1);
    'right: for x in (0..w).rev() {
        for y in top..=bot {
            if !is_white(pixels[y * w + x]) { right = x; break 'right; }
        }
    }

    let margin = 4usize;
    let top = top.saturating_sub(margin);
    let left = left.saturating_sub(margin);
    let bot = (bot + margin).min(h - 1);
    let right = (right + margin).min(w - 1);

    let cw = (right - left + 1) as u32;
    let ch = (bot - top + 1) as u32;
    let content_h = ch + TITLE_PAD;

    // Pad to 16:9 at a fixed output width
    let fixed_w: u32 = 2400;
    let fixed_h: u32 = fixed_w * 9 / 16; // 1350

    // Scale content to fit within the fixed frame (leaving room for title)
    let avail_h = fixed_h - TITLE_PAD;
    let scale_x = fixed_w as f32 / cw as f32;
    let scale_y = avail_h as f32 / ch as f32;
    let scale = scale_x.min(scale_y).min(1.0); // don't upscale

    let scaled_w = (cw as f32 * scale) as usize;
    let scaled_h = (ch as f32 * scale) as usize;

    let x_offset = (fixed_w as usize - scaled_w) / 2;
    let y_offset = TITLE_PAD as usize + (avail_h as usize - scaled_h) / 2;

    let mut out = resvg::tiny_skia::Pixmap::new(fixed_w, fixed_h)?;
    out.fill(resvg::tiny_skia::Color::WHITE);
    for dy in 0..scaled_h {
        for dx in 0..scaled_w {
            let sx = (dx as f32 / scale) as usize;
            let sy = (dy as f32 / scale) as usize;
            let sx = sx.min(cw as usize - 1);
            let sy = sy.min(ch as usize - 1);
            out.pixels_mut()[(y_offset + dy) * fixed_w as usize + (x_offset + dx)] =
                pixels[(top + sy) * w + (left + sx)];
        }
    }
    Some(out)
}

fn load_svg_to_pixmap(path: &Path) -> Option<resvg::tiny_skia::Pixmap> {
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
    let scale = 2400.0 / size.width();
    let w = (size.width() * scale) as u32;
    let h = (size.height() * scale) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    pixmap.fill(resvg::tiny_skia::Color::WHITE);
    let transform = resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    crop_pixmap(&pixmap)
}

fn load_raster_to_pixmap(path: &Path) -> Option<resvg::tiny_skia::Pixmap> {
    let img = image::open(path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();

    // Scale to 2400px wide to match SVG render width
    let target_w = 2400u32;
    let scale = target_w as f32 / w as f32;
    let target_h = (h as f32 * scale) as u32;
    let scaled = image::imageops::resize(&img, target_w, target_h, image::imageops::FilterType::Lanczos3);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)?;
    pixmap.fill(resvg::tiny_skia::Color::WHITE);
    for (i, pixel) in scaled.pixels().enumerate() {
        let x = i % target_w as usize;
        let y = i / target_w as usize;
        let src = resvg::tiny_skia::PremultipliedColorU8::from_rgba(
            pixel[0], pixel[1], pixel[2], pixel[3],
        );
        if let Some(src) = src {
            pixmap.pixels_mut()[y * target_w as usize + x] = src;
        }
    }
    crop_pixmap(&pixmap)
}

fn render_title_onto_pixmap(pixmap: &mut resvg::tiny_skia::Pixmap, slide: &Slide) {
    let w = pixmap.width();
    let font_size = 2400.0 / 40.0; // fixed size independent of image dimensions

    let verse_spans: String = slide
        .all_verses
        .iter()
        .map(|v| {
            let color = if *v == slide.current_verse { "black" } else { "grey" };
            format!(r#"<tspan fill="{color}">{v} </tspan>"#)
        })
        .collect();

    let title_escaped = slide.title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let title_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
            <text x="{cx}" y="{y}" font-family="serif" font-size="{fs}"
                  text-anchor="middle" fill="black">
                <tspan>{title}: </tspan>{verses}
            </text>
        </svg>"#,
        w = w,
        h = (font_size * 2.0) as u32,
        cx = w / 2,
        y = font_size * 1.3,
        fs = font_size,
        title = title_escaped,
        verses = verse_spans,
    );
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    if let Ok(tree) = resvg::usvg::Tree::from_data(title_svg.as_bytes(), &opt) {
        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    }
}

fn load_slide_texture(slide: &Slide) -> Option<gdk::Texture> {
    let ext = slide.path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let mut pixmap = if ext == "svg" {
        load_svg_to_pixmap(&slide.path)?
    } else {
        load_raster_to_pixmap(&slide.path)?
    };

    render_title_onto_pixmap(&mut pixmap, slide);

    let (w, h) = (pixmap.width(), pixmap.height());
    let bytes = glib::Bytes::from(pixmap.data());
    let texture = gdk::MemoryTexture::new(
        w as i32,
        h as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (w * 4) as usize,
    );
    Some(texture.upcast())
}

fn update_display(state: &AppState, picture: &gtk::Picture, nav_label: &gtk::Label) {
    if let Some(slide) = state.slides.get(state.current_slide) {
        if let Some(tex) = load_slide_texture(slide) {
            picture.set_paintable(Some(&tex));
        }
        nav_label.set_text(&format!("{}/{}", state.current_slide + 1, state.slides.len()));
    } else {
        picture.set_paintable(None::<&gdk::Texture>);
        nav_label.set_text("0/0");
    }
}

fn update_liturgy_label(state: &AppState, label: &gtk::Label) {
    if state.liturgy.is_empty() {
        label.set_text("");
    } else {
        let text: Vec<String> = state
            .liturgy
            .iter()
            .map(|e| {
                let vs = e.verses.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
                format!("{} ({})", e.song_name, vs)
            })
            .collect();
        label.set_text(&format!("Liturgy: {}", text.join(" | ")));
    }
}

fn rebuild_verse_checkboxes(
    verse_box: &gtk::FlowBox,
    verses: &[u32],
) {
    // Remove all children
    while let Some(child) = verse_box.first_child() {
        verse_box.remove(&child);
    }
    for &v in verses {
        let check = gtk::CheckButton::with_label(&format!("V{v}"));
        check.set_widget_name(&v.to_string());
        verse_box.insert(&check, -1);
    }
}

fn get_checked_verses(verse_box: &gtk::FlowBox) -> Vec<u32> {
    let mut verses = Vec::new();
    let mut child = verse_box.first_child();
    while let Some(widget) = child {
        if let Some(fb_child) = widget.downcast_ref::<gtk::FlowBoxChild>() {
            if let Some(check) = fb_child.child().and_then(|c| c.downcast::<gtk::CheckButton>().ok()) {
                if check.is_active() {
                    if let Ok(v) = check.widget_name().parse::<u32>() {
                        verses.push(v);
                    }
                }
            }
        }
        child = widget.next_sibling();
    }
    verses.sort();
    verses
}

fn set_all_checks(verse_box: &gtk::FlowBox, active: bool) {
    let mut child = verse_box.first_child();
    while let Some(widget) = child {
        if let Some(fb_child) = widget.downcast_ref::<gtk::FlowBoxChild>() {
            if let Some(check) = fb_child.child().and_then(|c| c.downcast::<gtk::CheckButton>().ok()) {
                check.set_active(active);
            }
        }
        child = widget.next_sibling();
    }
}

fn build_ui(app: &gtk::Application, cli: &CliArgs) {
    let state = Rc::new(RefCell::new(AppState::new(cli)));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Book of Praise")
        .default_width(1024)
        .default_height(768)
        .build();

    // --- Main layout: vertical box ---
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // --- Image display ---
    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_vexpand(true);
    picture.set_hexpand(true);
    picture.set_content_fit(gtk::ContentFit::Contain);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&picture));
    scroll.set_vexpand(true);

    vbox.append(&scroll);

    // --- Bottom controls ---
    let controls = gtk::Box::new(gtk::Orientation::Vertical, 4);
    controls.set_margin_start(8);
    controls.set_margin_end(8);
    controls.set_margin_top(4);
    controls.set_margin_bottom(8);

    // Liturgy label
    let liturgy_label = gtk::Label::new(None);
    liturgy_label.set_xalign(0.0);
    liturgy_label.set_wrap(true);
    controls.append(&liturgy_label);

    // Navigation row
    let nav_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let prev_btn = gtk::Button::with_label("Prev");
    let nav_label = gtk::Label::new(Some("0/0"));
    let next_btn = gtk::Button::with_label("Next");
    let clear_btn = gtk::Button::with_label("Clear liturgy");

    let png_label = gtk::Label::new(Some("PNG"));
    let svg_switch = gtk::Switch::new();
    svg_switch.set_active(!cli.png);
    let svg_label = gtk::Label::new(Some("SVG"));

    nav_row.append(&prev_btn);
    nav_row.append(&nav_label);
    nav_row.append(&next_btn);
    nav_row.append(&clear_btn);
    // Spacer to push toggle to the right
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    nav_row.append(&spacer);
    nav_row.append(&png_label);
    nav_row.append(&svg_switch);
    nav_row.append(&svg_label);
    controls.append(&nav_row);

    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    controls.append(&sep);

    // Song input row
    let input_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    input_row.set_valign(gtk::Align::Center);

    let psalm_radio = gtk::CheckButton::with_label("Psalm");
    let hymn_radio = gtk::CheckButton::with_label("Hymn");
    hymn_radio.set_group(Some(&psalm_radio));
    psalm_radio.set_active(true);

    let number_label = gtk::Label::new(Some("#"));
    let number_entry = gtk::Entry::new();
    number_entry.set_width_chars(5);

    let verse_box = gtk::FlowBox::new();
    verse_box.set_selection_mode(gtk::SelectionMode::None);
    verse_box.set_max_children_per_line(20);
    verse_box.set_hexpand(true);

    let all_btn = gtk::Button::with_label("All");
    let add_btn = gtk::Button::with_label("Add");

    input_row.append(&psalm_radio);
    input_row.append(&hymn_radio);
    input_row.append(&number_label);
    input_row.append(&number_entry);
    input_row.append(&verse_box);
    input_row.append(&all_btn);
    input_row.append(&add_btn);
    controls.append(&input_row);

    vbox.append(&controls);
    window.set_child(Some(&vbox));

    // --- Initial display ---
    update_display(&state.borrow(), &picture, &nav_label);
    update_liturgy_label(&state.borrow(), &liturgy_label);

    // --- Connect signals ---

    // Number entry changed -> update verse checkboxes
    {
        let state = state.clone();
        let verse_box = verse_box.clone();
        let psalm_radio = psalm_radio.clone();
        number_entry.connect_changed(move |entry| {
            let text = entry.text();
            if let Ok(num) = text.parse::<u32>() {
                let st = state.borrow();
                let song_type = if psalm_radio.is_active() { SongType::Psalm } else { SongType::Hymn };
                let verses = st.get_available_verses(song_type, num);
                rebuild_verse_checkboxes(&verse_box, &verses);
            } else {
                rebuild_verse_checkboxes(&verse_box, &[]);
            }
        });
    }

    // Radio toggle -> update verse checkboxes
    {
        let state = state.clone();
        let verse_box = verse_box.clone();
        let number_entry = number_entry.clone();
        let psalm_radio = psalm_radio.clone();
        psalm_radio.connect_active_notify(move |radio| {
            let text = number_entry.text();
            if let Ok(num) = text.parse::<u32>() {
                let st = state.borrow();
                let song_type = if radio.is_active() { SongType::Psalm } else { SongType::Hymn };
                let verses = st.get_available_verses(song_type, num);
                rebuild_verse_checkboxes(&verse_box, &verses);
            }
        });
    }

    // Prev
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        prev_btn.connect_clicked(move |_| {
            state.borrow_mut().navigate(-1);
            update_display(&state.borrow(), &picture, &nav_label);
        });
    }

    // Next
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        next_btn.connect_clicked(move |_| {
            state.borrow_mut().navigate(1);
            update_display(&state.borrow(), &picture, &nav_label);
        });
    }

    // Clear
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let liturgy_label = liturgy_label.clone();
        clear_btn.connect_clicked(move |_| {
            let mut st = state.borrow_mut();
            st.liturgy.clear();
            st.rebuild_slides();
            drop(st);
            update_display(&state.borrow(), &picture, &nav_label);
            update_liturgy_label(&state.borrow(), &liturgy_label);
        });
    }

    // SVG/PNG toggle
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let liturgy_label = liturgy_label.clone();
        let number_entry = number_entry.clone();
        let verse_box = verse_box.clone();
        svg_switch.connect_active_notify(move |switch| {
            let use_svg = switch.is_active();
            let mut st = state.borrow_mut();
            st.set_use_svg(use_svg);
            drop(st);
            number_entry.set_text("");
            rebuild_verse_checkboxes(&verse_box, &[]);
            update_display(&state.borrow(), &picture, &nav_label);
            update_liturgy_label(&state.borrow(), &liturgy_label);
        });
    }

    // All button
    {
        let state = state.clone();
        let verse_box = verse_box.clone();
        let number_entry = number_entry.clone();
        let psalm_radio = psalm_radio.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let liturgy_label = liturgy_label.clone();
        all_btn.connect_clicked(move |_| {
            set_all_checks(&verse_box, true);
            // Also add directly
            let text = number_entry.text();
            if let Ok(num) = text.parse::<u32>() {
                let song_type = if psalm_radio.is_active() { SongType::Psalm } else { SongType::Hymn };
                let verses = get_checked_verses(&verse_box);
                let mut st = state.borrow_mut();
                st.add_song(song_type, num, verses);
                drop(st);
                number_entry.set_text("");
                rebuild_verse_checkboxes(&verse_box, &[]);
                update_display(&state.borrow(), &picture, &nav_label);
                update_liturgy_label(&state.borrow(), &liturgy_label);
            }
        });
    }

    // Add button
    {
        let state = state.clone();
        let verse_box = verse_box.clone();
        let number_entry = number_entry.clone();
        let psalm_radio = psalm_radio.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let liturgy_label = liturgy_label.clone();
        add_btn.connect_clicked(move |_| {
            let text = number_entry.text();
            if let Ok(num) = text.parse::<u32>() {
                let song_type = if psalm_radio.is_active() { SongType::Psalm } else { SongType::Hymn };
                let verses = get_checked_verses(&verse_box);
                let mut st = state.borrow_mut();
                st.add_song(song_type, num, verses);
                drop(st);
                number_entry.set_text("");
                rebuild_verse_checkboxes(&verse_box, &[]);
                update_display(&state.borrow(), &picture, &nav_label);
                update_liturgy_label(&state.borrow(), &liturgy_label);
            }
        });
    }

    // Arrow key navigation
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gdk::Key::Left => {
                    state.borrow_mut().navigate(-1);
                    update_display(&state.borrow(), &picture, &nav_label);
                    glib::Propagation::Stop
                }
                gdk::Key::Right => {
                    state.borrow_mut().navigate(1);
                    update_display(&state.borrow(), &picture, &nav_label);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        window.add_controller(key_controller);
    }

    window.present();
}
