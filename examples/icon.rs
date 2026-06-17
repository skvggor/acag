//! Generates the app icon — a seigaiha "stamp" in the Omakase palette, no
//! Japanese glyphs. Writes `assets/icons/icon.svg` plus PNG sizes.
//! Run with: `cargo run --example icon`.

use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

const VIEWBOX: f32 = 512.0;

fn icon_svg() -> String {
    // A seigaiha wave "seal" in cream over a terracotta squircle — the house
    // mark, purely geometric. A big central fan with two sumi fans behind it
    // for depth, clipped to the rounded square.
    let fan = |cx: f32, cy: f32, base_r: f32, stroke: &str, width: f32, arcs: usize| {
        let mut paths = String::new();
        for index in 0..arcs {
            let radius = base_r * (1.0 - index as f32 / arcs as f32);
            paths.push_str(&format!(
                "<path d=\"M{:.1},{cy:.1} A{radius:.1},{radius:.1} 0 0 1 {:.1},{cy:.1}\"/>",
                cx - radius,
                cx + radius,
            ));
        }
        format!(
            "<g fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{width:.1}\" \
             stroke-linecap=\"round\">{paths}</g>"
        )
    };

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
         <g clip-path=\"url(#squircle)\">{back_left}{back_right}{front}</g>\
         </svg>",
        back_left = fan(118.0, 338.0, 188.0, "#1c130c", 22.0, 3),
        back_right = fan(394.0, 338.0, 188.0, "#1c130c", 22.0, 3),
        front = fan(256.0, 338.0, 210.0, "#f2e6d0", 30.0, 4),
    )
}

fn rasterize(svg: &str, size: u32) -> Vec<u8> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("valid icon svg");
    let mut pixmap = Pixmap::new(size, size).expect("pixmap");
    let scale = size as f32 / VIEWBOX;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().expect("encode png")
}

fn main() -> anyhow::Result<()> {
    let dir = Path::new("assets/icons");
    fs::create_dir_all(dir)?;
    let svg = icon_svg();
    fs::write(dir.join("icon.svg"), &svg)?;
    for size in [1024u32, 512, 256, 128] {
        fs::write(dir.join(format!("icon-{size}.png")), rasterize(&svg, size))?;
        println!("wrote icon-{size}.png");
    }
    println!("icons in {}", dir.display());
    Ok(())
}
