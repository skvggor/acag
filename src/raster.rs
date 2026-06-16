//! SVG → raster, via resvg (the native Rust engine `@resvg/resvg-js` only wraps).
//! The bundled Montserrat faces are registered in a shared `fontdb`, so the same
//! pipeline backs both the live preview and the 2160² PNG export.

use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, fontdb};

use crate::cover::CANVAS;

const FONT_REGULAR: &[u8] = include_bytes!("../assets/fonts/Montserrat-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/fonts/Montserrat-Bold.ttf");
const FONT_BLACK: &[u8] = include_bytes!("../assets/fonts/Montserrat-Black.ttf");

/// Default pixel size for the exported PNG.
pub const EXPORT_PIXELS: u32 = 2160;

fn shared_fontdb() -> Arc<fontdb::Database> {
    static DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_font_data(FONT_REGULAR.to_vec());
        db.load_font_data(FONT_BOLD.to_vec());
        db.load_font_data(FONT_BLACK.to_vec());
        db.set_sans_serif_family("Montserrat");
        Arc::new(db)
    })
    .clone()
}

fn parse(svg: &str) -> Result<usvg::Tree> {
    let options = usvg::Options {
        fontdb: shared_fontdb(),
        ..Default::default()
    };
    usvg::Tree::from_str(svg, &options).map_err(|error| anyhow!("invalid cover SVG: {error}"))
}

/// Rasterize the 1080-unit cover SVG into a square pixmap of `pixels` per side.
pub fn render_to_pixmap(svg: &str, pixels: u32) -> Result<Pixmap> {
    let tree = parse(svg)?;
    let mut pixmap =
        Pixmap::new(pixels, pixels).ok_or_else(|| anyhow!("invalid pixmap size {pixels}"))?;
    let scale = pixels as f32 / CANVAS as f32;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Rasterize and PNG-encode (used for the export).
pub fn png_bytes(svg: &str, pixels: u32) -> Result<Vec<u8>> {
    render_to_pixmap(svg, pixels)?
        .encode_png()
        .map_err(|error| anyhow!("PNG encoding failed: {error}"))
}

/// Straight-alpha RGBA8 for the slint preview (tiny-skia stores premultiplied).
pub fn render_to_rgba(svg: &str, pixels: u32) -> Result<(u32, u32, Vec<u8>)> {
    let pixmap = render_to_pixmap(svg, pixels)?;
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pixel in pixmap.data().chunks_exact(4) {
        let alpha = pixel[3];
        if alpha == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let unpremultiply = |c: u8| {
                ((u16::from(c) * 255 + u16::from(alpha) / 2) / u16::from(alpha)).min(255) as u8
            };
            rgba.extend_from_slice(&[
                unpremultiply(pixel[0]),
                unpremultiply(pixel[1]),
                unpremultiply(pixel[2]),
                alpha,
            ]);
        }
    }
    Ok((width, height, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover::{CoverConfig, render_cover_svg};

    #[test]
    fn rasterizes_default_cover_to_requested_size() {
        let svg = render_cover_svg(&CoverConfig::default());
        let pixmap = render_to_pixmap(&svg, 2160).expect("rasterize");
        assert_eq!(pixmap.width(), 2160);
        assert_eq!(pixmap.height(), 2160);
        // The background fills the canvas, so every pixel is opaque.
        assert!(pixmap.data().chunks_exact(4).all(|p| p[3] == 255));
    }

    #[test]
    fn png_bytes_have_png_signature() {
        let svg = render_cover_svg(&CoverConfig::default());
        let png = png_bytes(&svg, 256).expect("encode png");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn rgba_buffer_has_expected_length() {
        let svg = render_cover_svg(&CoverConfig::default());
        let (w, h, rgba) = render_to_rgba(&svg, 128).expect("rgba");
        assert_eq!((w, h), (128, 128));
        assert_eq!(rgba.len(), 128 * 128 * 4);
    }

    #[test]
    fn invalid_inputs_error_gracefully() {
        let svg = render_cover_svg(&CoverConfig::default());
        assert!(render_to_pixmap(&svg, 0).is_err()); // zero-size pixmap
        assert!(render_to_pixmap("definitely not svg", 64).is_err()); // parse error
    }
}
