//! Bloco — constructivist: a solid accent column on the left holding a large
//! number over wagara texture, the title set big to the right. Strong vertical
//! tension and a block of color. The number sits on the colored column, so its
//! color is forced to AAA against the accent.

use crate::cover::elements::{
    Anchor, brand, divider, kicker, number, readable, textured_panel, title_block,
};
use crate::cover::render::Ctx;

pub fn render(ctx: &Ctx) -> String {
    let margin = 96.0;
    let column_width = 340.0;
    let right_x = column_width + 72.0;
    let right_width = ctx.size - right_x - margin;
    let mut svg = String::new();

    svg.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{column_width:.1}\" height=\"{:.1}\" fill=\"{fill}\"/>",
        ctx.size,
        fill = ctx.theme.accent.to_hex(),
    ));

    // Readable ink over the colored column (exercises the AAA guardrail).
    let on_column = readable(ctx.theme.bg, ctx.theme.accent);
    svg.push_str(&textured_panel(
        ctx,
        0.0,
        0.0,
        column_width,
        ctx.size,
        on_column,
        0.16,
        "col-tex",
    ));
    svg.push_str(&number(
        ctx,
        column_width / 2.0,
        margin + 90.0,
        Anchor::Middle,
        on_column,
        64.0,
    ));

    let kicker_baseline = margin + 60.0;
    svg.push_str(&kicker(ctx, right_x, kicker_baseline));

    let title = title_block(
        ctx,
        right_x,
        kicker_baseline + 130.0,
        right_width,
        118.0,
        54.0,
        5,
        Anchor::Start,
        false,
    );
    svg.push_str(&title.svg);

    let footer_baseline = ctx.size - margin - 8.0;
    svg.push_str(&divider(ctx, right_x, footer_baseline - 46.0, 240.0));
    svg.push_str(&brand(ctx, right_x, footer_baseline, Anchor::Start));
    svg
}
