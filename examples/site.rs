//! Generates the landing page's static imagery with the app's own engine, the
//! way `gallery.rs` and `icon.rs` do: a 1200×630 Open Graph card (a real cover
//! in the 1.91:1 link format), the poster cover used as the no-JS / no-WASM
//! fallback, the favicon / PWA icon sizes, and the Montserrat webfont copies.
//! Deterministic — the same configs always produce the same files.
//!
//! Run with: `cargo run --example site --no-default-features --features render`

use std::fs;
use std::path::Path;

use anyhow::Result;

use article_cover_art_generator::cover::config::CoverConfig;
use article_cover_art_generator::cover::format::Format;
use article_cover_art_generator::cover::layouts::Layout;
use article_cover_art_generator::cover::render_cover_svg;
use article_cover_art_generator::design::patterns::Pattern;
use article_cover_art_generator::design::themes::ThemeName;
use article_cover_art_generator::raster;

/// The cover shown before the live engine takes over — mirrored by
/// `FIRST_PLATE` in `web/js/main.js`, so the swap to the live render is
/// invisible.
fn poster_config() -> CoverConfig {
    CoverConfig {
        title: "Design systems that scale".to_owned(),
        category: "engineering".to_owned(),
        date: String::new(),
        number: "014".to_owned(),
        brand: "skvggor.dev".to_owned(),
        theme: ThemeName::Terracotta,
        pattern: Pattern::Seigaiha,
        layout: Layout::Editorial,
        format: Format::Square,
        grain: 0.25,
        pattern_strength: 1.0,
    }
}

/// The Open Graph card is itself a cover in the link format — exactly the
/// artwork the app would export for sharing this page.
fn og_config() -> CoverConfig {
    CoverConfig {
        title: "Article cover art, plated by the house".to_owned(),
        category: "omakase".to_owned(),
        date: String::new(),
        number: String::new(),
        brand: "github.com/skvggor/acag".to_owned(),
        theme: ThemeName::Terracotta,
        pattern: Pattern::Seigaiha,
        layout: Layout::Editorial,
        format: Format::Social,
        grain: 0.25,
        pattern_strength: 1.0,
    }
}

/// Rasterize the app's own logo SVG into the favicon / PWA icon sizes the page
/// links.
fn write_icons(root: &Path) -> Result<()> {
    let icons = root.join("web/assets/icons");
    fs::create_dir_all(&icons)?;
    let logo = fs::read_to_string(root.join("assets/icons/icon.svg"))?;
    for size in [32u32, 180, 192, 512] {
        let path = icons.join(format!("icon-{size}.png"));
        fs::write(&path, raster::png_bytes(&logo, size)?)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// The injected cover SVG asks for Montserrat by family name, so the page
/// serves the same faces the engine measures with.
fn write_fonts(root: &Path) -> Result<()> {
    let fonts = root.join("web/assets/fonts");
    fs::create_dir_all(&fonts)?;
    for weight in ["Regular", "Bold", "Black"] {
        let name = format!("Montserrat-{weight}.ttf");
        fs::copy(root.join("assets/fonts").join(&name), fonts.join(&name))?;
        println!("wrote {}", fonts.join(&name).display());
    }
    Ok(())
}

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img = root.join("web/assets/img");
    fs::create_dir_all(&img)?;

    let og = raster::png_bytes(&render_cover_svg(&og_config()), 1200)?;
    fs::write(img.join("og.png"), &og)?;
    println!("wrote {}", img.join("og.png").display());

    let poster = raster::png_bytes(&render_cover_svg(&poster_config()), 1080)?;
    fs::write(img.join("poster.png"), &poster)?;
    println!("wrote {}", img.join("poster.png").display());

    write_icons(root)?;
    write_fonts(root)?;
    Ok(())
}
