//! Editorial — asymmetric: kicker top-left, a wagara seal roundel bleeding off
//! the top-right, title anchored bottom-left under an accent brush mark, brand
//! and divider in the footer. The most faithful to the user's sites.

use crate::cover::elements::{
    Anchor, accent_tick, brand, divider, kicker, number, seal_roundel, title_block,
};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let margin = 96.0;
    let mut svg = String::new();

    svg.push_str(&seal_roundel(
        ctx,
        ctx.size - 120.0,
        230.0,
        300.0,
        ctx.theme.line,
        0.16,
    ));

    let kicker_baseline = margin + 60.0;
    svg.push_str(&kicker(ctx, margin, kicker_baseline));
    svg.push_str(&number(
        ctx,
        margin,
        kicker_baseline + 58.0,
        Anchor::Start,
        ctx.theme.text,
        30.0,
    ));

    let footer_baseline = ctx.size - margin - 8.0;
    let title = title_block(
        ctx,
        margin,
        footer_baseline - 120.0,
        ctx.size - 2.0 * margin,
        150.0,
        64.0,
        4,
        Anchor::Start,
        true,
    );
    svg.push_str(&accent_tick(ctx, margin, title.top - 40.0, 120.0));
    svg.push_str(&title.svg);

    svg.push_str(&divider(ctx, margin, footer_baseline - 46.0, 300.0));
    svg.push_str(&brand(ctx, margin, footer_baseline, Anchor::Start));
    svg
}
