//! Reusable SVG building blocks composed differently by each layout: kicker,
//! brand, divider, number, the wagara "seal", textured panels, and the
//! auto-fitted title block. All readable text is forced through [`readable`]
//! so it clears WCAG AAA (7:1) against its immediate background.

use crate::cover::render::Ctx;
use crate::cover::typeset::Weight;
use crate::design::contrast::{Rgb, ensure_readable};

pub fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Adjust `fg` until it reaches AAA contrast against `bg`.
pub fn readable(fg: Rgb, bg: Rgb) -> Rgb {
    ensure_readable(fg, bg, 7.0)
}

#[derive(Clone, Copy)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    fn svg(self) -> &'static str {
        match self {
            Anchor::Start => "start",
            Anchor::Middle => "middle",
            Anchor::End => "end",
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn text_el(
    x: f64,
    baseline: f64,
    content: &str,
    size: f64,
    weight: Weight,
    color: Rgb,
    anchor: Anchor,
    letter_spacing: f64,
) -> String {
    format!(
        "<text x=\"{x:.1}\" y=\"{baseline:.1}\" font-family=\"Montserrat\" \
         font-weight=\"{weight}\" font-size=\"{size:.1}\" fill=\"{fill}\" \
         text-anchor=\"{anchor}\" letter-spacing=\"{letter_spacing:.2}\">{content}</text>",
        weight = weight.css(),
        fill = color.to_hex(),
        anchor = anchor.svg(),
        content = esc(content),
    )
}

/// A clipped rectangle filled with the chosen wagara pattern — used for the
/// faint full-canvas texture and the solid accent column.
#[allow(clippy::too_many_arguments)]
pub fn textured_panel(
    ctx: &Ctx,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: Rgb,
    opacity: f64,
    id: &str,
) -> String {
    let scale = ctx.cfg.pattern.default_scale();
    let shapes = ctx.cfg.pattern.shapes(w, h, scale);
    let opacity = opacity * ctx.cfg.pattern_strength;
    format!(
        "<clipPath id=\"{id}\"><rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\"/></clipPath>\
         <g clip-path=\"url(#{id})\"><g transform=\"translate({x:.1},{y:.1})\" fill=\"none\" \
         stroke=\"{stroke}\" stroke-width=\"2\" opacity=\"{opacity:.3}\">{shapes}</g></g>",
        stroke = color.to_hex(),
    )
}

/// A circular "stamp" of the wagara pattern — the geometric seal that replaces
/// the kana seal from the other projects.
pub fn seal_roundel(ctx: &Ctx, cx: f64, cy: f64, r: f64, color: Rgb, opacity: f64) -> String {
    let scale = ctx.cfg.pattern.default_scale();
    let shapes = ctx.cfg.pattern.shapes(2.0 * r, 2.0 * r, scale);
    let opacity = opacity * ctx.cfg.pattern_strength;
    let id = format!("seal-{}", cx as i64);
    format!(
        "<clipPath id=\"{id}\"><circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"{r:.1}\"/></clipPath>\
         <g clip-path=\"url(#{id})\"><g transform=\"translate({x:.1},{y:.1})\" fill=\"none\" \
         stroke=\"{stroke}\" stroke-width=\"3\" opacity=\"{opacity:.3}\">{shapes}</g></g>",
        x = cx - r,
        y = cy - r,
        stroke = color.to_hex(),
    )
}

/// Uppercase, letter-spaced kicker with a short accent tick, e.g.
/// `▍ ENGINEERING · 16 JUN 2026`.
pub fn kicker(ctx: &Ctx, x: f64, baseline: f64) -> String {
    let mut parts = Vec::new();
    if !ctx.cfg.category.is_empty() {
        parts.push(ctx.cfg.category.to_uppercase());
    }
    if !ctx.cfg.date.is_empty() {
        parts.push(ctx.cfg.date.to_uppercase());
    }
    if parts.is_empty() {
        return String::new();
    }
    let label = parts.join("   ·   ");
    let tick_w = 64.0;
    let tick_h = 8.0;
    let tick = format!(
        "<rect x=\"{x:.1}\" y=\"{ty:.1}\" width=\"{tick_w:.1}\" height=\"{tick_h:.1}\" fill=\"{fill}\"/>",
        ty = baseline - tick_h,
        fill = ctx.theme.accent.to_hex(),
    );
    let text = text_el(
        x + tick_w + 24.0,
        baseline,
        &label,
        22.0,
        Weight::Bold,
        ctx.theme.text,
        Anchor::Start,
        4.0,
    );
    format!("{tick}{text}")
}

pub fn brand(ctx: &Ctx, x: f64, baseline: f64, anchor: Anchor) -> String {
    if ctx.cfg.brand.is_empty() {
        return String::new();
    }
    text_el(
        x,
        baseline,
        &ctx.cfg.brand,
        24.0,
        Weight::Bold,
        ctx.theme.text,
        anchor,
        2.0,
    )
}

pub fn number(ctx: &Ctx, x: f64, baseline: f64, anchor: Anchor, color: Rgb, size: f64) -> String {
    if ctx.cfg.number.is_empty() {
        return String::new();
    }
    let label = format!("Nº {}", ctx.cfg.number);
    text_el(x, baseline, &label, size, Weight::Black, color, anchor, 1.0)
}

pub fn divider(ctx: &Ctx, x: f64, y: f64, w: f64) -> String {
    format!(
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"3\" fill=\"{fill}\" opacity=\"0.5\"/>",
        fill = ctx.theme.line.to_hex(),
    )
}

/// Film-grain overlay: fractal noise desaturated to gray and laid over the whole
/// canvas at `intensity` in `[0, 1]`. Uses resvg's `feTurbulence` /
/// `feColorMatrix` support, so it shows identically in preview and export.
pub fn grain_overlay(size: f64, intensity: f64) -> String {
    let opacity = intensity.clamp(0.0, 1.0) * 0.22;
    format!(
        "<filter id=\"grain\" x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" \
         color-interpolation-filters=\"sRGB\">\
         <feTurbulence type=\"fractalNoise\" baseFrequency=\"0.85\" numOctaves=\"2\" \
         stitchTiles=\"stitch\"/>\
         <feColorMatrix type=\"saturate\" values=\"0\"/>\
         </filter>\
         <rect width=\"{size:.0}\" height=\"{size:.0}\" filter=\"url(#grain)\" opacity=\"{opacity:.3}\"/>"
    )
}

/// A short, thick brush mark in the accent hue — the chromatic punch that keeps
/// the title itself monochrome (and therefore AAA).
pub fn accent_tick(ctx: &Ctx, x: f64, y: f64, w: f64) -> String {
    format!(
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"10\" fill=\"{fill}\"/>",
        fill = ctx.theme.accent.to_hex(),
    )
}

pub struct TitleBlock {
    pub svg: String,
    /// Approximate cap-height top of the first line.
    pub top: f64,
    pub size: f64,
}

/// Auto-fit and render the title. `anchor_y` is the first baseline when
/// `bottom_anchored` is false, otherwise the last baseline.
#[allow(clippy::too_many_arguments)]
pub fn title_block(
    ctx: &Ctx,
    x: f64,
    anchor_y: f64,
    max_width: f64,
    max_size: f64,
    min_size: f64,
    max_lines: usize,
    align: Anchor,
    bottom_anchored: bool,
) -> TitleBlock {
    let (size, lines) = ctx
        .black
        .fit(&ctx.cfg.title, max_width, max_size, min_size, max_lines);
    let line_height = size * 1.06;
    let count = lines.len() as f64;
    let first_baseline = if bottom_anchored {
        anchor_y - (count - 1.0) * line_height
    } else {
        anchor_y
    };
    let mut svg = String::new();
    for (index, line) in lines.iter().enumerate() {
        let baseline = first_baseline + index as f64 * line_height;
        svg.push_str(&text_el(
            x,
            baseline,
            line,
            size,
            Weight::Black,
            ctx.theme.text,
            align,
            0.0,
        ));
    }
    TitleBlock {
        svg,
        top: first_baseline - size * 0.74,
        size,
    }
}
