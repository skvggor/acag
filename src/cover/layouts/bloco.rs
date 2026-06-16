//! Bloco — constructivist: a solid accent column on the left holding a large
//! number over wagara texture, the title set big to the right. Strong vertical
//! tension and a block of color. The number sits on the colored column, so its
//! color is forced to AAA against the accent.

use crate::cover::elements::{
    Anchor, brand, divider, kicker, number, readable, textured_panel, title_block,
};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let m = ctx.margin();
    let (w, h) = (ctx.width, ctx.height);
    let column_width = (w * 0.3).clamp(220.0, 480.0);
    let right_x = column_width + (m * 0.8).max(48.0);
    let right_width = w - right_x - m;
    let mut svg = String::new();

    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{column_width:.1}\" height=\"{h:.1}\" fill=\"{fill}\"/>",
        fill = ctx.theme.accent.to_hex(),
    ));

    // Readable ink over the colored column (exercises the AAA guardrail).
    let on_column = readable(ctx.theme.bg, ctx.theme.accent);
    svg.push_str(&textured_panel(
        ctx,
        0.0,
        0.0,
        column_width,
        h,
        on_column,
        0.16,
        "col-tex",
    ));
    svg.push_str(&number(
        ctx,
        column_width / 2.0,
        m + 70.0,
        Anchor::Middle,
        on_column,
        (column_width * 0.18).clamp(40.0, 72.0),
    ));

    let kicker_baseline = m + 44.0;
    svg.push_str(&kicker(ctx, right_x, kicker_baseline));

    let footer_baseline = h - m + 4.0;
    let title_top = kicker_baseline + 36.0;
    let max_height = (footer_baseline - 56.0 - title_top).max(80.0);
    let max_size = (h * 0.14).clamp(54.0, 132.0);
    let title = title_block(
        ctx,
        right_x,
        title_top,
        right_width,
        max_height,
        max_size,
        36.0,
        Anchor::Start,
        false,
    );
    svg.push_str(&title.svg);

    svg.push_str(&divider(
        ctx,
        right_x,
        footer_baseline - 42.0,
        (right_width * 0.4).clamp(160.0, 300.0),
    ));
    svg.push_str(&brand(ctx, right_x, footer_baseline, Anchor::Start));
    svg
}
