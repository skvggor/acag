//! Ma (間) — maximum negative space: a kicker, a large title breathing near the
//! top, a faint wagara seal in the bottom-right corner, and number/brand pinned
//! to the footer corners. Adapts to any aspect ratio.

use crate::cover::elements::{Anchor, brand, kicker, number, seal_roundel, title_block};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let m = ctx.margin() * 1.15;
    let (w, h) = (ctx.width, ctx.height);
    let mut svg = String::new();

    // Faint seal in the bottom-right corner.
    let seal_r = ctx.min_side() * 0.3;
    svg.push_str(&seal_roundel(
        ctx,
        w - seal_r * 0.25,
        h - seal_r * 0.25,
        seal_r,
        ctx.theme.line,
        0.12,
    ));

    let kicker_baseline = m + 30.0;
    svg.push_str(&kicker(ctx, m, kicker_baseline));

    let title_top = kicker_baseline + 44.0;
    let footer_baseline = h - m + 16.0;
    // Leave generous Ma below the title.
    let max_height = ((footer_baseline - 80.0 - title_top) * 0.82).max(80.0);
    let max_width = w - 2.0 * m;
    let max_size = (h * 0.16).clamp(60.0, 150.0);
    let title = title_block(
        ctx,
        m,
        title_top,
        max_width,
        max_height,
        max_size,
        38.0,
        Anchor::Start,
        false,
    );
    svg.push_str(&title.svg);

    svg.push_str(&number(
        ctx,
        m,
        footer_baseline,
        Anchor::Start,
        ctx.theme.text,
        28.0,
    ));
    svg.push_str(&brand(ctx, w - m, footer_baseline, Anchor::End));
    svg
}
