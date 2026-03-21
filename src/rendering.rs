use std::path::Path;

use gtk4 as gtk;
use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;

use crate::model::Slide;
use crate::render_ly;

pub const DEFAULT_RENDER_WIDTH: u32 = 2400;

pub type Pixmap = resvg::tiny_skia::Pixmap;

/// Parse and rasterize an SVG file into a pixmap, scaling to `render_width` pixels
/// while preserving aspect ratio. Replaces `currentColor` with black for compatibility.
pub fn load_svg_pixmap(path: &Path, render_width: u32) -> Option<Pixmap> {
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

/// Load a PNG image and scale it to `render_width` pixels using Lanczos3 filtering.
pub fn load_png_pixmap(path: &Path, render_width: u32) -> Option<Pixmap> {
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
pub fn crop_and_frame(src: &Pixmap, render_width: u32) -> Option<Pixmap> {
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

/// Render a song title and verse indicator into the top region of the pixmap
/// using an inline SVG overlay. The current verse is shown in black, others in grey.
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

/// Load a slide's image (SVG or PNG), crop whitespace, frame it in 16:9 with a
/// title overlay, and return a GDK texture ready for display.
pub fn load_slide_texture(slide: &Slide, render_width: u32) -> Option<gdk::Texture> {
    let is_svg = slide
        .path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));

    let raw = if is_svg {
        // Resolve the actual cached SVG path
        let cached = render_ly::svg_path_for_verse(&slide.song_dir, slide.current_verse)?;
        load_svg_pixmap(&cached, render_width)?
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
