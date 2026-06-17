//! Editorial — asymmetric: kicker top-left, a wagara seal roundel bleeding off
//! the top-right, title anchored bottom-left under an accent brush mark, brand
//! and divider in the footer. Adapts to any aspect ratio.

use crate::cover::elements::{
    Anchor, accent_tick, brand, divider, kicker, number, seal_roundel, title_block,
};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let m = ctx.margin();
    let (w, h) = (ctx.width, ctx.height);
    let mut svg = String::new();

    // Seal roundel bleeding from the top-right corner.
    let seal_r = ctx.min_side() * 0.34;
    svg.push_str(&seal_roundel(
        ctx,
        w - m,
        m + seal_r * 0.35,
        seal_r,
        ctx.theme.line,
        0.16,
    ));

    let kicker_baseline = m + 44.0;
    svg.push_str(&kicker(ctx, m, kicker_baseline));
    svg.push_str(&number(
        ctx,
        m,
        kicker_baseline + 52.0,
        Anchor::Start,
        ctx.theme.text,
        28.0,
    ));

    let footer_baseline = h - m + 4.0;
    let title_bottom = footer_baseline - 64.0;
    let title_top_limit = kicker_baseline + 84.0;
    let max_height = (title_bottom - title_top_limit).max(80.0);
    let max_width = w - 2.0 * m;
    let max_size = (h * 0.16).clamp(64.0, 150.0);
    let title = title_block(
        ctx,
        m,
        title_bottom,
        max_width,
        max_height,
        max_size,
        38.0,
        Anchor::Start,
        true,
    );
    svg.push_str(&accent_tick(ctx, m, title.top - 32.0, 110.0));
    svg.push_str(&title.svg);

    svg.push_str(&divider(
        ctx,
        m,
        footer_baseline - 42.0,
        (w * 0.28).clamp(180.0, 360.0),
    ));
    svg.push_str(&brand(ctx, m, footer_baseline, Anchor::Start));
    svg
}
