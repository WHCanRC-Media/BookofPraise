use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;

use crate::model::{
    base_dir, read_verify_count, increment_verify, AppState, SongLibrary, SongType,
};
use crate::lyric_check;
use crate::render_ly;
use crate::rendering::{current_png_path, load_slide_texture, save_current_png, DEFAULT_RENDER_WIDTH};
use crate::updater::{has_changes, check_for_update, collect_pr_files, create_pr_with_files, download_and_extract, generate_branch_name, should_report_hymn_usage, email_hymn_usage};

// ── UI helpers ──────────────────────────────────────────────────────

/// Populate the editor panel's text views and entry fields from the current slide's
/// LilyPond source files (`notes.ly`, `lyrics_N.ly`, `song.yaml`).
fn load_editor_contents(state: &AppState, notes_view: &gtk::TextView, lyrics_view: &gtk::TextView, notes_label: &gtk::Label, lyrics_label: &gtk::Label, split_style_dropdown: &gtk::DropDown) {
    if let Some(slide) = state.slides.get(state.current_slide) {
        let notes_path = render_ly::notes_path_for_verse(&slide.song_dir, slide.current_verse);
        let lyrics_path = slide.song_dir.join(format!("lyrics_{}.ly", slide.current_verse));

        let notes_text = std::fs::read_to_string(&notes_path).unwrap_or_default();
        notes_view.buffer().set_text(&notes_text);

        let lyrics_text = std::fs::read_to_string(&lyrics_path).unwrap_or_default();
        lyrics_view.buffer().set_text(&lyrics_text);

        let meta = render_ly::read_song_meta(&slide.song_dir);
        let style_idx = render_ly::SplitStyle::ALL.iter()
            .position(|s| *s == meta.split_style)
            .unwrap_or(0);
        split_style_dropdown.set_selected(style_idx as u32);

        notes_label.set_text(notes_path.file_name().unwrap_or_default().to_str().unwrap_or("notes.ly"));
        lyrics_label.set_text(&format!("lyrics_{}.ly", slide.current_verse));
    } else {
        notes_view.buffer().set_text("");
        lyrics_view.buffer().set_text("");
        split_style_dropdown.set_selected(0);
        notes_label.set_text("notes.ly");
        lyrics_label.set_text("lyrics.ly");
    }
}

/// Snapshot all patchable files (.ly, song.yaml) in a song directory to a temp
/// directory, so we can produce a real diff later. Only copies on first edit.
fn snapshot_originals(state: &mut AppState, song_dir: &std::path::Path) {
    // Create temp dir on first use
    if state.originals_dir.is_none() {
        if let Ok(td) = tempfile::tempdir() {
            state.originals_dir = Some(td);
        } else {
            return;
        }
    }
    let tmp_base = state.originals_dir.as_ref().unwrap().path();
    let dir_name = song_dir.file_name().unwrap_or_default();
    let snap_dir = tmp_base.join(dir_name);
    if snap_dir.exists() {
        return; // already snapshotted
    }
    let _ = std::fs::create_dir_all(&snap_dir);
    if let Ok(entries) = std::fs::read_dir(song_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".ly") || name == "song.yaml" {
                let _ = std::fs::copy(entry.path(), snap_dir.join(&name));
            }
        }
    }
}

/// Write the editor panel's current text back to the corresponding LilyPond
/// source files on disk and mark the song directory as edited.
fn save_editor_contents(state: &mut AppState, notes_view: &gtk::TextView, lyrics_view: &gtk::TextView, split_style_dropdown: &gtk::DropDown) {
    let slide_info = state.slides.get(state.current_slide)
        .map(|s| (s.song_dir.clone(), s.current_verse));
    if let Some((song_dir, verse)) = slide_info {
        snapshot_originals(state, &song_dir);
        state.edited_song_dirs.insert(song_dir.clone());
        let notes_path = render_ly::notes_path_for_verse(&song_dir, verse);
        let lyrics_path = song_dir.join(format!("lyrics_{verse}.ly"));

        let buf = notes_view.buffer();
        let notes_text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let _ = std::fs::write(&notes_path, notes_text.as_str());

        let buf = lyrics_view.buffer();
        let lyrics_text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let _ = std::fs::write(&lyrics_path, lyrics_text.as_str());

        let style_idx = split_style_dropdown.selected() as usize;
        let split_style = render_ly::SplitStyle::ALL.get(style_idx)
            .cloned()
            .unwrap_or_default();
        let mut meta = render_ly::read_song_meta(&song_dir);
        meta.split_style = split_style;
        render_ly::write_song_meta(&song_dir, &meta);
    }
}

/// Invalidate all verses of a song: clear cached textures/errors for any
/// slides in that song. The SVG cache is content-addressed so stale entries
/// are harmless — new content will hash to a different filename.
/// Save editor contents, detect what changed, and invalidate the appropriate caches.
fn save_and_invalidate(
    state: &mut AppState,
    notes_view: &gtk::TextView,
    lyrics_view: &gtk::TextView,
    split_style_dropdown: &gtk::DropDown,
) {
    let (notes_changed, split_changed) = if let Some(slide) = state.slides.get(state.current_slide) {
        let notes_path = render_ly::notes_path_for_verse(&slide.song_dir, slide.current_verse);
        let old = std::fs::read_to_string(&notes_path).unwrap_or_default();
        let buf = notes_view.buffer();
        let new = buf.text(&buf.start_iter(), &buf.end_iter(), false);

        let old_meta = render_ly::read_song_meta(&slide.song_dir);
        let new_style_idx = split_style_dropdown.selected() as usize;
        let new_style = render_ly::SplitStyle::ALL.get(new_style_idx)
            .cloned()
            .unwrap_or_default();
        (old != new.as_str(), old_meta.split_style != new_style)
    } else {
        (false, false)
    };

    save_editor_contents(state, notes_view, lyrics_view, split_style_dropdown);

    if notes_changed || split_changed {
        // Notes or split style changed — part count may differ,
        // so invalidate and rebuild the slide list.
        if let Some(slide) = state.slides.get(state.current_slide) {
            let song_dir = slide.song_dir.clone();
            invalidate_song(state, &song_dir);
        }
        state.rebuild_slides();
    } else if let Some(slide) = state.slides.get(state.current_slide) {
        // Only invalidate current verse/part
        let song_dir = slide.song_dir.clone();
        let path = slide.path.clone();
        let verse = slide.current_verse;
        let part = slide.part;
        render_ly::invalidate_combined_cache(&song_dir);
        state.texture_cache.retain(|k, _| k.0 != path || k.1 != verse || k.2 != part);
        state.render_errors.remove(&(path.clone(), verse, part));
        let _ = std::fs::remove_file(&path);
    }
}

fn invalidate_song(state: &mut AppState, song_dir: &std::path::Path) {
    render_ly::invalidate_combined_cache(song_dir);
    let keys: Vec<_> = state.slides.iter()
        .filter(|sl| sl.song_dir == song_dir)
        .map(|sl| (sl.path.clone(), sl.current_verse, sl.part))
        .collect();
    for (path, verse, part) in keys {
        state.texture_cache.retain(|k, _| k.0 != path || k.1 != verse || k.2 != part);
        state.render_errors.remove(&(path, verse, part));
    }
}

/// Return `true` if the slide is an SVG that has no up-to-date cached render.
fn needs_render(slide: &crate::model::Slide) -> bool {
    let is_svg = slide
        .path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    is_svg && !render_ly::is_svg_current(&slide.song_dir, slide.current_verse, slide.part)
}

/// Spawn a background LilyPond render for the given song_dir/verse/part.
/// Only touches state — no UI widgets needed. When the render completes,
/// updates state and chains to the next unrendered slide.
fn start_render(
    state_rc: &Rc<RefCell<AppState>>,
    song_dir: std::path::PathBuf,
    verse: u32,
    part: u32,
) {
    if !render_ly::lilypond_available() {
        return;
    }
    let render_key = (song_dir.clone(), verse, part);
    {
        let mut state = state_rc.borrow_mut();
        if state.rendering.contains(&render_key) {
            return;
        }
        if render_ly::is_svg_current(&song_dir, verse, part) {
            return;
        }
        state.rendering.insert(render_key.clone());
    }

    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let result = render_ly::render_svg(&song_dir, verse, part);
        let _ = tx.send(result);
    });

    let state_rc2 = state_rc.clone();
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
                prefetch_next(&state_rc2);
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(_) => glib::ControlFlow::Break,
        }
    });
}

/// Find the next slide that needs rendering and start it. Searches forward
/// from the current slide, wrapping around, so all slides are eventually rendered.
fn prefetch_next(state_rc: &Rc<RefCell<AppState>>) {
    let state = state_rc.borrow();
    let n = state.slides.len();
    if n == 0 {
        return;
    }
    let start = state.current_slide;
    for offset in 1..n {
        let i = (start + offset) % n;
        let slide = &state.slides[i];
        if !needs_render(slide) {
            continue;
        }
        let key = (slide.song_dir.clone(), slide.current_verse, slide.part);
        if state.rendering.contains(&key) || state.render_errors.contains_key(&key) {
            continue;
        }
        let song_dir = slide.song_dir.clone();
        let verse = slide.current_verse;
        let part = slide.part;
        drop(state);
        start_render(state_rc, song_dir, verse, part);
        return;
    }
}

/// Update the main image display for the current slide. Loads a cached texture
/// if available, otherwise spawns a background LilyPond render and polls for
/// completion. Also updates the navigation label, verify button, and error state.
fn refresh_display(
    state_rc: &Rc<RefCell<AppState>>,
    picture: &gtk::Picture,
    nav_label: &gtk::Label,
    spinner: &gtk::Spinner,
    error_label: &gtk::Label,
    verify_btn: &gtk::Button,
    mismatch_label: &gtk::Label,
) {
    let mut state = state_rc.borrow_mut();
    spinner.stop();
    spinner.set_visible(false);
    error_label.set_visible(false);
    mismatch_label.set_visible(false);
    mismatch_label.set_text("");

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
            (slide.path.clone(), slide.current_verse, slide.part, render_width),
            (slide.song_dir.clone(), slide.current_verse, slide.part),
            slide.song_dir.clone(),
            slide.current_verse,
            slide.part,
            state.current_slide,
            state.slides.len(),
        )
    });

    if let Some((cache_key, render_key, song_dir, verse, part, idx, total)) = slide_info {
        nav_label.set_text(&format!("{}/{}", idx + 1, total));

        // Update verify button state
        let verify_count = read_verify_count(&song_dir, verse);
        if verify_count >= 1 {
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
            save_current_png(&tex);
            picture.set_paintable(Some(&tex));
            state.texture_cache.insert(cache_key, tex);
            if let Some(svg_path) = render_ly::svg_path_for_verse(&song_dir, verse, part) {
                let n_parts = render_ly::num_parts_for_verse(&song_dir, verse);
                let pairs = lyric_check::mismatch_pairs_nospace(
                    &song_dir, verse, part, &svg_path, n_parts,
                );
                if !pairs.is_empty() {
                    let text = pairs
                        .iter()
                        .map(|(s, v)| format!("src: {s}\nsvg: {v}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    mismatch_label.set_text(&text);
                    mismatch_label.set_visible(true);
                }
            }
            drop(state);
            prefetch_next(state_rc);
            return;
        }

        // Need to render — start it if not already running
        if needs_render(&state.slides[idx]) && !state.rendering.contains(&render_key) {
            drop(state);
            start_render(state_rc, song_dir, verse, part);
        } else {
            drop(state);
        }

        // Show spinner and poll until the render completes (only if lilypond is available)
        if render_ly::lilypond_available() {
            spinner.set_visible(true);
            spinner.start();
            picture.set_paintable(None::<&gdk::Texture>);

            let state_rc2 = state_rc.clone();
            let picture2 = picture.clone();
            let nav_label2 = nav_label.clone();
            let spinner2 = spinner.clone();
            let error_label2 = error_label.clone();
            let verify_btn2 = verify_btn.clone();
            let mismatch_label2 = mismatch_label.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                let is_done = {
                    let state = state_rc2.borrow();
                    !state.rendering.contains(&render_key)
                };
                if is_done {
                    refresh_display(&state_rc2, &picture2, &nav_label2, &spinner2, &error_label2, &verify_btn2, &mismatch_label2);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }
    } else {
        picture.set_paintable(None::<&gdk::Texture>);
        nav_label.set_text("0/0");
        verify_btn.set_label("Verify");
        verify_btn.set_sensitive(false);
    }
}

/// Update the liturgy summary label with the current list of songs and verses.
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

/// Clear and repopulate the verse checkbox row with one `CheckButton` per verse.
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

/// Collect the verse numbers of all currently checked checkboxes, sorted.
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

/// Set all verse checkboxes in the flow box to active.
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

/// Construct the main application window and wire up all widgets and signal handlers.
/// This is the top-level UI entry point called once on application activation.
pub fn build_ui(app: &gtk::Application, cli: &crate::model::Cli) {
    // Load help-button CSS
    let css = gtk::CssProvider::new();
    css.load_from_data(
        ".help-btn { border-radius: 50%; border: 1px solid @theme_fg_color; min-width: 18px; min-height: 18px; padding: 0; margin: 0; font-weight: bold; font-size: 12px; } .help-btn label { margin: 0; padding: 0; margin-top: -1px; }",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("display"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

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

    // Lyric-mismatch label (red, wraps, anchored to bottom-center)
    let mismatch_label = gtk::Label::new(None);
    mismatch_label.set_wrap(true);
    mismatch_label.set_selectable(true);
    mismatch_label.set_halign(gtk::Align::Center);
    mismatch_label.set_valign(gtk::Align::End);
    mismatch_label.set_margin_start(24);
    mismatch_label.set_margin_end(24);
    mismatch_label.set_margin_bottom(8);
    mismatch_label.set_visible(false);
    mismatch_label.add_css_class("lyric-mismatch");
    let mismatch_css = gtk::CssProvider::new();
    mismatch_css.load_from_data(
        ".lyric-mismatch { color: red; font-family: monospace; background: rgba(255,255,255,0.85); padding: 4px 8px; border-radius: 4px; }",
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("display"),
        &mismatch_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&picture));
    overlay.add_overlay(&spinner);
    overlay.add_overlay(&error_label);
    overlay.add_overlay(&mismatch_label);

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
    let notes_help_btn = gtk::Button::with_label("?");
    notes_help_btn.add_css_class("help-btn");
    notes_help_btn.set_tooltip_text(Some("Note syntax help"));
    let notes_label_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    notes_label_row.append(&notes_label);
    notes_label_row.append(&notes_help_btn);

    let lyrics_label = gtk::Label::new(Some("lyrics.ly"));
    lyrics_label.set_xalign(0.0);
    let lyrics_help_btn = gtk::Button::with_label("?");
    lyrics_help_btn.add_css_class("help-btn");
    lyrics_help_btn.set_tooltip_text(Some("Lyrics syntax help"));
    let lyrics_label_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    lyrics_label_row.append(&lyrics_label);
    lyrics_label_row.append(&lyrics_help_btn);

    let split_style_label = gtk::Label::new(Some("Split style"));
    split_style_label.set_xalign(0.0);
    let split_style_items: Vec<&str> = render_ly::SplitStyle::ALL.iter()
        .map(|s| s.label())
        .collect();
    let split_style_dropdown = gtk::DropDown::from_strings(&split_style_items);
    split_style_dropdown.set_selected(0);

    let save_btn = gtk::Button::with_label("Save & Re-render");
    let revert_btn = gtk::Button::with_label("Revert");

    let btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_row.append(&save_btn);
    btn_row.append(&revert_btn);

    let editor_panel = gtk::Box::new(gtk::Orientation::Vertical, 4);
    editor_panel.set_margin_start(4);
    editor_panel.set_margin_end(4);
    editor_panel.set_margin_top(4);
    editor_panel.set_margin_bottom(4);
    editor_panel.append(&split_style_label);
    editor_panel.append(&split_style_dropdown);
    editor_panel.append(&notes_label_row);
    editor_panel.append(&notes_scroll);
    editor_panel.append(&lyrics_label_row);
    editor_panel.append(&lyrics_scroll);
    editor_panel.append(&btn_row);
    editor_panel.set_visible(false);
    editor_panel.set_hexpand(true);

    {
        let window = window.clone();
        lyrics_help_btn.connect_clicked(move |_| {
            let dialog = gtk::MessageDialog::new(
                Some(&window),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Info,
                gtk::ButtonsType::Ok,
                "",
            );
            dialog.set_markup(
                "<b>Editing Lyrics</b>\n\n\
                 <b>Syllables:</b> Separate syllables with spaces.\n\
                 Each syllable aligns to one note.\n\n\
                 <b>Hyphens:</b> Use <tt> -- </tt> (space-dash-dash-space)\n\
                 between syllables of the same word.\n\
                 Example: <tt>A -- ma -- zing grace</tt>\n\n\
                 <b>Multi-note syllables (melisma):</b>\n\
                 Use <tt>_</tt> after a syllable to extend it\n\
                 over additional notes (adds an extender line).\n\
                 Example: <tt>grace _ _ how sweet</tt>\n\
                 This holds \"grace\" across 3 notes.\n\n\
                 <b>Skipping notes:</b> Use <tt>_</tt> alone to\n\
                 skip a note without any lyric.\n\n\
                 <b>Tied notes:</b> When notes are tied together,\n\
                 they count as one note for lyrics — write\n\
                 the syllable only once."
            );
            dialog.connect_response(|dlg, _| {
                dlg.close();
            });
            dialog.show();
        });
    }

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
    let lyrics_mag_label = gtk::Label::new(Some("Lyrics ×"));
    let lyrics_mag_spin = gtk::SpinButton::with_range(0.5, 3.0, 0.1);
    lyrics_mag_spin.set_digits(1);
    lyrics_mag_spin.set_value(render_ly::lyrics_magnification());
    lyrics_mag_spin.set_tooltip_text(Some("Lyrics font-size magnification"));
    lyrics_mag_spin.set_visible(!cli.png);
    lyrics_mag_label.set_visible(!cli.png);

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
        lyrics_mag_label.upcast_ref(),
        lyrics_mag_spin.upcast_ref(),
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

    // Path label at bottom
    let path_label = gtk::Label::new(None);
    path_label.set_xalign(0.0);
    path_label.set_selectable(true);
    path_label.set_margin_start(4);
    path_label.set_margin_end(4);
    let png_path = current_png_path();
    path_label.set_text(&png_path.to_string_lossy());

    // Main layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.append(&hpaned);
    vbox.append(&controls);
    vbox.append(&path_label);
    window.set_child(Some(&vbox));

    // Initial display
    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
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
    connect!(prev_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, editor_panel, notes_view, lyrics_view, notes_label, lyrics_label, split_style_dropdown], move |_| {
        state.borrow_mut().navigate(-1);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
        if editor_panel.is_visible() {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
        }
    });
    connect!(next_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, editor_panel, notes_view, lyrics_view, notes_label, lyrics_label, split_style_dropdown], move |_| {
        state.borrow_mut().navigate(1);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
        if editor_panel.is_visible() {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
        }
    });

    // Clear
    connect!(clear_btn, connect_clicked, [state, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, liturgy_label], move |_| {
        { let mut s = state.borrow_mut(); s.liturgy.clear(); s.rebuild_slides(); }
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
        refresh_liturgy(&state.borrow(), &liturgy_label);
    });

    // Edit toggle
    connect!(edit_btn, connect_toggled, [state, editor_panel, notes_view, lyrics_view, notes_label, lyrics_label, split_style_dropdown], move |btn| {
        let active = btn.is_active();
        editor_panel.set_visible(active);
        if active {
            load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
        }
    });

    // Verify button
    {
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let spinner = spinner.clone();
        let error_label = error_label.clone();
        let mismatch_label = mismatch_label.clone();
        let verify_btn2 = verify_btn.clone();
        verify_btn.connect_clicked(move |_| {
            {
                let s = state.borrow();
                if let Some(slide) = s.slides.get(s.current_slide) {
                    increment_verify(&slide.song_dir, slide.current_verse);
                }
            }
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn2, &mismatch_label);
        });
    }

    // Notes syntax help
    connect!(notes_help_btn, connect_clicked, [window], move |_| {
        let help_text = "\
<b>Notes</b>
Notes are written as letter names: <tt>c d e f g a b</tt>
Follow with a duration number: <tt>c4</tt> (quarter), <tt>d2</tt> (half), <tt>e1</tt> (whole), <tt>f8</tt> (eighth)
Use <tt>r</tt> for rests: <tt>r4</tt> (quarter rest)

<b>Octaves</b>
Notes use relative mode — each note is placed closest to the previous one.
To force a note up an octave, add <tt>'</tt> (apostrophe): <tt>c'</tt>
To force a note down an octave, add <tt>,</tt> (comma): <tt>c,</tt>
These stack: <tt>c''</tt> = two octaves up

<b>Sharps and Flats</b>
Sharp: add <tt>is</tt> after the note name: <tt>fis</tt> (F#), <tt>cis</tt> (C#)
Flat: add <tt>es</tt> after the note name: <tt>bes</tt> (Bb), <tt>ees</tt> (Eb)

<b>Slurs</b>
Put <tt>(</tt> after the first note and <tt>)</tt> after the last note:
<tt>f8( ees8)</tt> — two slurred eighth notes";
        let dialog = gtk::MessageDialog::new(
            Some(&window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Info,
            gtk::ButtonsType::Close,
            "",
        );
        dialog.set_title(Some("Note Syntax Help"));
        dialog.set_markup(help_text);
        dialog.connect_response(|dlg, _| dlg.close());
        dialog.show();
    });

    // Save & Re-render
    connect!(save_btn, connect_clicked, [state, notes_view, lyrics_view, split_style_dropdown, picture, nav_label, spinner, error_label, verify_btn, mismatch_label], move |_| {
        save_and_invalidate(&mut state.borrow_mut(), &notes_view, &lyrics_view, &split_style_dropdown);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
    });

    // Revert to original files from snapshot
    connect!(revert_btn, connect_clicked, [state, notes_view, lyrics_view, notes_label, lyrics_label, split_style_dropdown, picture, nav_label, spinner, error_label, verify_btn, mismatch_label], move |_| {
        {
            let mut s = state.borrow_mut();
            if let Some(slide) = s.slides.get(s.current_slide) {
                let song_dir = slide.song_dir.clone();
                if let Some(ref tmp) = s.originals_dir {
                    let dir_name = song_dir.file_name().unwrap_or_default();
                    let snap_dir = tmp.path().join(dir_name);
                    if snap_dir.exists() {
                        // Copy original files back
                        if let Ok(entries) = std::fs::read_dir(&snap_dir) {
                            for entry in entries.flatten() {
                                let name = entry.file_name();
                                let _ = std::fs::copy(entry.path(), song_dir.join(&name));
                            }
                        }
                        invalidate_song(&mut s, &song_dir);
                    }
                }
            }
            load_editor_contents(&s, &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
        }
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
    });

    // Lyrics magnification
    connect!(lyrics_mag_spin, connect_value_changed, [state, picture, nav_label, spinner, error_label, verify_btn, mismatch_label], move |spin| {
        render_ly::set_lyrics_magnification(spin.value());
        render_ly::invalidate_all_combined_cache();
        {
            let mut s = state.borrow_mut();
            s.texture_cache.clear();
            s.render_errors.clear();
        }
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
    });

    // SVG/PNG toggle
    connect!(svg_switch, connect_active_notify, [state, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, liturgy_label, number_entry, verse_box, edit_btn, editor_panel, lyrics_mag_label, lyrics_mag_spin], move |sw| {
        state.borrow_mut().set_use_svg(sw.is_active());
        edit_btn.set_active(false);
        edit_btn.set_visible(sw.is_active());
        verify_btn.set_visible(sw.is_active());
        lyrics_mag_label.set_visible(sw.is_active());
        lyrics_mag_spin.set_visible(sw.is_active());
        editor_panel.set_visible(false);
        number_entry.set_text("");
        rebuild_verse_checks(&verse_box, &[]);
        refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
        refresh_liturgy(&state.borrow(), &liturgy_label);
    });

    // All button — check all and add immediately
    connect!(all_btn, connect_clicked, [state, verse_box, number_entry, song_type, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, liturgy_label], move |_| {
        check_all(&verse_box);
        if let Ok(num) = number_entry.text().parse::<u32>() {
            let verses = checked_verses(&verse_box);
            state.borrow_mut().add_song_with_verses(song_type(), num, verses);
            number_entry.set_text("");
            rebuild_verse_checks(&verse_box, &[]);
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
            refresh_liturgy(&state.borrow(), &liturgy_label);
        }
    });

    // Add button
    connect!(add_btn, connect_clicked, [state, verse_box, number_entry, song_type, picture, nav_label, spinner, error_label, verify_btn, mismatch_label, liturgy_label], move |_| {
        if let Ok(num) = number_entry.text().parse::<u32>() {
            let verses = checked_verses(&verse_box);
            state.borrow_mut().add_song_with_verses(song_type(), num, verses);
            number_entry.set_text("");
            rebuild_verse_checks(&verse_box, &[]);
            refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
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
        let mismatch_label = mismatch_label.clone();
        let verify_btn = verify_btn.clone();
        let editor_panel = editor_panel.clone();
        let notes_view = notes_view.clone();
        let lyrics_view = lyrics_view.clone();
        let notes_label = notes_label.clone();
        let lyrics_label = lyrics_label.clone();
        let split_style_dropdown = split_style_dropdown.clone();
        let kc = gtk::EventControllerKey::new();
        kc.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk::Key::s && modifiers.contains(gdk::ModifierType::CONTROL_MASK) && editor_panel.is_visible() {
                save_and_invalidate(&mut state.borrow_mut(), &notes_view, &lyrics_view, &split_style_dropdown);
                refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
                return glib::Propagation::Stop;
            }
            match key {
                gdk::Key::Left => {
                    state.borrow_mut().navigate(-1);
                    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
                    if editor_panel.is_visible() {
                        load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Right => {
                    state.borrow_mut().navigate(1);
                    refresh_display(&state, &picture, &nav_label, &spinner, &error_label, &verify_btn, &mismatch_label);
                    if editor_panel.is_visible() {
                        load_editor_contents(&state.borrow(), &notes_view, &lyrics_view, &notes_label, &lyrics_label, &split_style_dropdown);
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        window.add_controller(kc);
    }

    // ── Offer to submit edits as PR on close ──
    {
        let state = state.clone();
        window.connect_close_request(move |win| {
            let s = state.borrow();
            let edited = s.edited_song_dirs.clone();
            let originals_path = s.originals_dir.as_ref().map(|td| td.path().to_path_buf());
            drop(s);
            if edited.is_empty() || !has_changes(&edited, originals_path.as_deref()) {
                return glib::Propagation::Proceed;
            }

            if crate::updater::GITHUB_PAT.is_empty() {
                let dialog = gtk::MessageDialog::new(
                    Some(win),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    "This app was built without the ability to share updates.",
                );
                let win = win.clone();
                dialog.connect_response(move |dlg, _| {
                    dlg.close();
                    win.destroy();
                });
                dialog.show();
                return glib::Propagation::Stop;
            }

            let dialog = gtk::MessageDialog::new(
                Some(win),
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Question,
                gtk::ButtonsType::YesNo,
                "You made edits this session. Submit your corrections for review?",
            );
            let win = win.clone();
            dialog.connect_response(move |dlg, resp| {
                dlg.close();
                if resp == gtk::ResponseType::Yes {
                    let files = collect_pr_files(&edited);
                    let branch = generate_branch_name();
                    let dir_names: Vec<String> = edited
                        .iter()
                        .filter_map(|d| d.file_name().map(|n| n.to_string_lossy().to_string()))
                        .collect();
                    let title = format!("BOP edits: {}", dir_names.join(", "));
                    let body = "Edits from the Book of Praise application.".to_string();
                    match create_pr_with_files("master", &branch, &files, &title, &body) {
                        Ok(url) => eprintln!("PR created: {url}"),
                        Err(e) => eprintln!("Failed to create PR: {e}"),
                    }
                }
                win.destroy();
            });
            dialog.show();
            glib::Propagation::Stop
        });
    }

    window.present();

    // Closure for dialogs that should only appear after LilyPond is confirmed
    // available, so that multiple modal dialogs don't stack on top of each other.
    let run_post_lilypond_dialogs: Rc<dyn Fn()> = {
        let window = window.clone();
        let state = state.clone();
        let picture = picture.clone();
        let nav_label = nav_label.clone();
        let spinner = spinner.clone();
        let error_label = error_label.clone();
        let mismatch_label = mismatch_label.clone();
        let verify_btn = verify_btn.clone();
        let cli_update = cli.update;
        Rc::new(move || {
            // Closure for the update check, called after the hymn dialog (if any) is dismissed.
            let run_update_check: Rc<dyn Fn()> = {
                let window = window.clone();
                let state = state.clone();
                let picture = picture.clone();
                let nav_label = nav_label.clone();
                let spinner = spinner.clone();
                let error_label = error_label.clone();
        let mismatch_label = mismatch_label.clone();
                let verify_btn = verify_btn.clone();
                Rc::new(move || {
                    if !cli_update { return; }
                    let window = window.clone();
                    let state = state.clone();
                    let picture = picture.clone();
                    let nav_label = nav_label.clone();
                    let spinner = spinner.clone();
                    let error_label = error_label.clone();
        let mismatch_label = mismatch_label.clone();
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
                                    &format!("Song data update available: {tag}\nDownload and install?"),
                                );
                                let window2 = window.clone();
                                let state2 = state.clone();
                                let picture2 = picture.clone();
                                let nav_label2 = nav_label.clone();
                                let spinner2 = spinner.clone();
                                let error_label2 = error_label.clone();
                let mismatch_label2 = mismatch_label.clone();
                                let verify_btn2 = verify_btn.clone();
                                let asset_url = Rc::new(asset_url);
                                dialog.connect_response(move |dlg, resp| {
                                    dlg.close();
                                    if resp == gtk::ResponseType::Yes {
                                        let asset_url = (*asset_url).clone();
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
                                        let mismatch_label3 = mismatch_label2.clone();
                                        let verify_btn3 = verify_btn2.clone();
                                        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                                            match rx2.try_recv() {
                                                Ok(Ok(())) => {
                                                    progress.close();
                                                    {
                                                        let mut s = state3.borrow_mut();
                                                        s.songs_dir = base_dir(s.use_svg);
                                                        s.library = SongLibrary::scan(&s.songs_dir);
                                                        s.texture_cache.clear();
                                                        s.render_errors.clear();
                                                        s.rebuild_slides();
                                                    }
                                                    refresh_display(&state3, &picture3, &nav_label3, &spinner3, &error_label3, &verify_btn3, &mismatch_label3);
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
                            Ok(_) => glib::ControlFlow::Break,
                            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                            Err(_) => glib::ControlFlow::Break,
                        }
                    });
                })
            };

            // ── Hymn usage report (new year) ──
            if should_report_hymn_usage() {
                let dialog = gtk::Dialog::with_buttons(
                    Some("Hymn Usage Report"),
                    Some(&window),
                    gtk::DialogFlags::MODAL,
                    &[("Send", gtk::ResponseType::Accept), ("Skip", gtk::ResponseType::Cancel)],
                );
                let content = dialog.content_area();
                content.set_spacing(8);
                content.set_margin_start(12);
                content.set_margin_end(12);
                content.set_margin_top(12);
                content.set_margin_bottom(12);

                let label = gtk::Label::new(Some(
                    "It\u{2019}s a new year. Would you like to email last year\u{2019}s hymn usage report?",
                ));
                label.set_wrap(true);
                content.append(&label);

                let entry_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let email_label = gtk::Label::new(Some("Email:"));
                let email_entry = gtk::Entry::new();
                email_entry.set_hexpand(true);
                email_entry.set_placeholder_text(Some("recipient@example.com"));
                entry_box.append(&email_label);
                entry_box.append(&email_entry);
                content.append(&entry_box);

                dialog.connect_response(move |dlg, resp| {
                    dlg.close();
                    if resp == gtk::ResponseType::Accept {
                        let addr = email_entry.text().to_string();
                        if !addr.is_empty() {
                            if let Err(e) = email_hymn_usage(&addr) {
                                eprintln!("Failed to send hymn usage report: {e}");
                            }
                        }
                    }
                    run_update_check();
                });
                dialog.show();
            } else {
                run_update_check();
            }
        })
    };

    // ── Ensure LilyPond is available ──
    if !render_ly::lilypond_available() {
        let dialog = gtk::MessageDialog::new(
            Some(&window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Question,
            gtk::ButtonsType::None,
            "Download GNU LilyPond from the internet? (required)",
        );
        dialog.add_button("Exit", gtk::ResponseType::Close);
        dialog.add_button("Yes", gtk::ResponseType::Yes);
        let window2 = window.clone();
        let state2 = state.clone();
        let picture2 = picture.clone();
        let nav_label2 = nav_label.clone();
        let spinner2 = spinner.clone();
        let error_label2 = error_label.clone();
                let mismatch_label2 = mismatch_label.clone();
        let verify_btn2 = verify_btn.clone();
        dialog.connect_response(move |dlg, resp| {
            if resp == gtk::ResponseType::Yes {
                dlg.set_text(Some("Downloading LilyPond..."));
                dlg.set_response_sensitive(gtk::ResponseType::Yes, false);
                dlg.set_response_sensitive(gtk::ResponseType::Close, false);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(render_ly::download_lilypond());
                });
                let dlg2 = dlg.clone();
                let window3 = window2.clone();
                let state3 = state2.clone();
                let picture3 = picture2.clone();
                let nav_label3 = nav_label2.clone();
                let spinner3 = spinner2.clone();
                let error_label3 = error_label2.clone();
                                        let mismatch_label3 = mismatch_label2.clone();
                let verify_btn3 = verify_btn2.clone();
                let post_dialogs = run_post_lilypond_dialogs.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                    match rx.try_recv() {
                        Ok(Ok(())) => {
                            dlg2.hide();
                            dlg2.destroy();
                            refresh_display(&state3, &picture3, &nav_label3, &spinner3, &error_label3, &verify_btn3, &mismatch_label3);
                            post_dialogs();
                            glib::ControlFlow::Break
                        }
                        Ok(Err(e)) => {
                            dlg2.close();
                            eprintln!("LilyPond download failed: {e}");
                            window3.destroy();
                            glib::ControlFlow::Break
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(_) => { dlg2.close(); window3.destroy(); glib::ControlFlow::Break }
                    }
                });
            } else {
                window2.destroy();
            }
        });
        dialog.show();
    } else {
        run_post_lilypond_dialogs();
    }
}
