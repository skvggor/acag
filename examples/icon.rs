//! Generates the app icon — an "a" monogram logomark in Montserrat Black over a
//! full-bleed seigaiha wave background, in the Omakase palette. Writes
//! `assets/icons/icon.svg` plus PNG sizes. Run with: `cargo run --example icon`.

use std::fs;
use std::path::Path;

use article_cover_art_generator::raster;

/// 青海波 — overlapping seigaiha wave fans (concentric arcs), the same motif the
/// app's first pattern draws, generated over a `size`×`size` area.
fn seigaiha(size: f32, scale: f32) -> String {
    let radii = [scale, scale * 2.0 / 3.0, scale / 3.0];
    let mut d = String::new();
    let (mut row, mut y) = (0u32, 0.0);
    while y <= size + scale {
        let offset = if row % 2 == 1 { scale } else { 0.0 };
        let mut x = -scale + offset;
        while x <= size + scale {
            for r in radii {
                d.push_str(&format!(
                    "M{:.1},{y:.1} A{r:.1},{r:.1} 0 0 0 {:.1},{y:.1} ",
                    x - r,
                    x + r,
                ));
            }
            x += 2.0 * scale;
        }
        row += 1;
        y += scale;
    }
    d.trim_end().to_string()
}

fn icon_svg() -> String {
    // A logomark: a lowercase "a" in Montserrat Black (the app's typeface), cream,
    // centered over a full-bleed seigaiha wave texture on a terracotta squircle.
    const INK: &str = "#2a1810";
    const CREAM: &str = "#f2e6d0";

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"512\" height=\"512\" \
         viewBox=\"0 0 512 512\">\
         <defs>\
         <linearGradient id=\"bg\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">\
         <stop offset=\"0\" stop-color=\"#cf7b45\"/><stop offset=\"1\" stop-color=\"#9a4a22\"/>\
         </linearGradient>\
         <clipPath id=\"squircle\"><rect width=\"512\" height=\"512\" rx=\"116\"/></clipPath>\
         </defs>\
         <rect width=\"512\" height=\"512\" rx=\"116\" fill=\"url(#bg)\"/>\
         <g clip-path=\"url(#squircle)\">\
         <g fill=\"none\" stroke=\"{INK}\" stroke-width=\"5\" stroke-opacity=\"0.18\">\
         <path d=\"{waves}\"/></g>\
         <text x=\"256\" y=\"345\" text-anchor=\"middle\" font-family=\"Montserrat\" \
         font-weight=\"900\" font-size=\"340\" fill=\"{CREAM}\">a</text>\
         </g>\
         </svg>",
        waves = seigaiha(512.0, 64.0),
    )
}

fn main() -> anyhow::Result<()> {
    let dir = Path::new("assets/icons");
    fs::create_dir_all(dir)?;
    let svg = icon_svg();
    fs::write(dir.join("icon.svg"), &svg)?;
    for size in [1024u32, 512, 256, 128] {
        fs::write(
            dir.join(format!("icon-{size}.png")),
            raster::png_bytes(&svg, size)?,
        )?;
        println!("wrote icon-{size}.png");
    }

    // Multi-resolution .ico embedded into the Windows .exe (see build.rs).
    let mut ico = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [256u32, 128, 64, 48, 32, 16] {
        let png = raster::png_bytes(&svg, size)?;
        let image = ico::IconImage::read_png(&png[..])?;
        ico.add_entry(ico::IconDirEntry::encode(&image)?);
    }
    ico.write(fs::File::create(dir.join("icon.ico"))?)?;
    println!("wrote icon.ico");

    println!("icons in {}", dir.display());
    Ok(())
}
