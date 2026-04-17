#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lyric_check;
mod model;
mod render_ly;
mod rendering;
mod ui;
mod updater;

use clap::Parser;
use gtk4 as gtk;
#[cfg(target_os = "windows")]
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use std::rc::Rc;

use model::Cli;

// ── Entry point ─────────────────────────────────────────────────────

/// Application entry point. Parses CLI arguments, builds a GTK application,
/// and launches the main UI window.
fn main() -> glib::ExitCode {
    // Disable client-side decorations so the native Windows titlebar is used.
    #[cfg(target_os = "windows")]
    // SAFETY: called before any other threads are spawned.
    unsafe { std::env::set_var("GTK_CSD", "0") };

    let cli = Cli::parse();
    if cli.update {
        model::enable_data_dir_mode();
    }
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
    app.connect_activate(move |app| ui::build_ui(app, &cli));
    app.run_with_args::<&str>(&[])
}
