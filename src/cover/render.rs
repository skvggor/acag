//! Orchestrates a full 1080×1080 cover SVG: background, faint wagara texture,
//! the chosen layout's foreground, all in one string. This pure function is the
//! single source of truth — the live preview and the PNG export both rasterize
//! exactly what it returns.

use crate::cover::config::CoverConfig;
use crate::cover::elements::{grain_overlay, textured_panel};
use crate::cover::layouts::{Layout, bloco, editorial, ma};
use crate::cover::typeset::{Font, Weight};
use crate::design::themes::Theme;

/// Side of the square design canvas, in user units. The PNG is rasterized at a
/// higher pixel size from this resolution-independent SVG.
pub const CANVAS: f64 = 1080.0;

/// Everything a layout needs to compose its foreground.
pub struct Ctx<'a> {
    pub cfg: &'a CoverConfig,
    pub theme: Theme,
    pub size: f64,
    /// Montserrat Black, for measuring/fitting the title.
    pub black: &'a Font,
}

pub fn render_cover_svg(cfg: &CoverConfig) -> String {
    let theme = cfg.theme.palette();
    let black = Font::new(Weight::Black);
    let ctx = Ctx {
        cfg,
        theme,
        size: CANVAS,
        black: &black,
    };

    let texture = textured_panel(&ctx, 0.0, 0.0, CANVAS, CANVAS, theme.line, 0.05, "bg-tex");
    let foreground = match cfg.layout {
        Layout::Editorial => editorial::render(&ctx),
        Layout::Bloco => bloco::render(&ctx),
        Layout::Ma => ma::render(&ctx),
    };
    let grain = if cfg.grain {
        grain_overlay(CANVAS)
    } else {
        String::new()
    };

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{CANVAS}\" height=\"{CANVAS}\" \
         viewBox=\"0 0 {CANVAS} {CANVAS}\">\
         <rect width=\"{CANVAS}\" height=\"{CANVAS}\" fill=\"{bg}\"/>{texture}{foreground}{grain}</svg>",
        bg = theme.bg.to_hex(),
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]
    use super::*;
    use crate::design::themes::ThemeName;

    fn has_cjk(text: &str) -> bool {
        text.chars().any(|c| {
            let u = c as u32;
            (0x3000..=0x303F).contains(&u)
                || (0x3040..=0x309F).contains(&u)
                || (0x30A0..=0x30FF).contains(&u)
                || (0x4E00..=0x9FFF).contains(&u)
                || (0xFF00..=0xFFEF).contains(&u)
        })
    }

    #[test]
    fn renders_valid_square_svg_with_content() {
        let cfg = CoverConfig::default();
        let svg = render_cover_svg(&cfg);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("viewBox=\"0 0 1080 1080\""));
        assert!(svg.ends_with("</svg>"));
        // The title is auto-wrapped into one <text> per line, so check words.
        assert!(svg.contains("Design") && svg.contains("scale"));
        assert!(svg.contains("skvggor.dev")); // brand
        assert!(svg.contains("Nº 014")); // number
        assert!(svg.to_uppercase().contains("ENGINEERING")); // kicker
    }

    #[test]
    fn never_emits_japanese_glyphs() {
        // The hard rule: identity comes from geometry/color only.
        let mut cfg = CoverConfig::default();
        for theme in ThemeName::ALL {
            for layout in Layout::ALL {
                cfg.theme = theme;
                cfg.layout = layout;
                assert!(!has_cjk(&render_cover_svg(&cfg)), "CJK leaked into the SVG");
            }
        }
    }

    #[test]
    fn theme_changes_background_color() {
        let mut cfg = CoverConfig::default();
        cfg.theme = ThemeName::Terracotta;
        let terracotta = render_cover_svg(&cfg);
        cfg.theme = ThemeName::Ai;
        let ai = render_cover_svg(&cfg);
        assert!(terracotta.contains("#d89868"));
        assert!(ai.contains("#1b3a4b"));
        assert_ne!(terracotta, ai);
    }

    #[test]
    fn every_layout_renders_distinct_non_empty_svg() {
        let mut cfg = CoverConfig::default();
        let mut outputs = Vec::new();
        for layout in Layout::ALL {
            cfg.layout = layout;
            let svg = render_cover_svg(&cfg);
            assert!(svg.len() > 200);
            outputs.push(svg);
        }
        assert_ne!(outputs[0], outputs[1]);
        assert_ne!(outputs[1], outputs[2]);
        assert_ne!(outputs[0], outputs[2]);
    }

    #[test]
    fn optional_fields_are_hidden_when_empty() {
        let mut cfg = CoverConfig::default();
        cfg.brand = String::new();
        cfg.number = String::new();
        cfg.category = String::new();
        cfg.date = String::new();
        let svg = render_cover_svg(&cfg);
        assert!(!svg.contains("skvggor.dev"));
        assert!(!svg.contains("Nº"));
        // Title is required and still present.
        assert!(svg.contains("Design"));
    }
}
