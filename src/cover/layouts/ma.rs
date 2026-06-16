//! Ma (間) — maximum negative space: a kicker, a large title breathing near the
//! top under an accent brush mark, a faint wagara seal in the bottom-right
//! corner, and number/brand pinned to the footer corners.

use crate::cover::elements::{Anchor, brand, kicker, number, seal_roundel, title_block};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let margin = 120.0;
    let mut svg = String::new();

    svg.push_str(&seal_roundel(
        ctx,
        ctx.size - 80.0,
        ctx.size - 80.0,
        210.0,
        ctx.theme.line,
        0.12,
    ));

    let kicker_baseline = margin + 30.0;
    svg.push_str(&kicker(ctx, margin, kicker_baseline));

    let title = title_block(
        ctx,
        margin,
        kicker_baseline + 170.0,
        ctx.size - 2.0 * margin,
        144.0,
        72.0,
        4,
        Anchor::Start,
        false,
    );
    svg.push_str(&title.svg);

    let footer_baseline = ctx.size - margin + 16.0;
    svg.push_str(&number(
        ctx,
        margin,
        footer_baseline,
        Anchor::Start,
        ctx.theme.text,
        30.0,
    ));
    svg.push_str(&brand(ctx, ctx.size - margin, footer_baseline, Anchor::End));
    svg
}
