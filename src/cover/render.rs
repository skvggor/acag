//! Orchestrates a full cover SVG (any aspect ratio): background, faint wagara
//! texture, the chosen layout's foreground, all in one string. This pure
//! function is the single source of truth — the live preview and the PNG export
//! both rasterize exactly what it returns.

use crate::cover::config::CoverConfig;
use crate::cover::elements::{grain_overlay, textured_panel};
use crate::cover::layouts::{Layout, bloco, editorial, ma};
use crate::cover::typeset::{Font, Weight};
use crate::design::themes::Theme;

/// Everything a layout needs to compose its foreground.
pub struct Ctx<'a> {
    pub cfg: &'a CoverConfig,
    pub theme: Theme,
    pub width: f64,
    pub height: f64,
    /// Montserrat Black, for measuring/fitting the title.
    pub black: &'a Font,
}

impl Ctx<'_> {
    /// The smaller canvas dimension — used for margins and seal sizing so they
    /// stay sensible across aspect ratios.
    pub fn min_side(&self) -> f64 {
        self.width.min(self.height)
    }

    /// A margin proportional to the smaller side, with a floor.
    pub fn margin(&self) -> f64 {
        (self.min_side() * 0.075).max(56.0)
    }
}

pub fn render_cover_svg(cfg: &CoverConfig) -> String {
    thread_local! {
        // Parsing the Montserrat Black face is reused across renders.
        static TITLE_FONT: Font = Font::new(Weight::Black);
    }

    TITLE_FONT.with(|black| {
        let theme = cfg.theme.palette();
        let (width, height) = cfg.format.dimensions();
        let ctx = Ctx {
            cfg,
            theme,
            width,
            height,
            black,
        };

        let texture = textured_panel(&ctx, 0.0, 0.0, width, height, theme.line, 0.05, "bg-tex");
        let foreground = match cfg.layout {
            Layout::Editorial => editorial::render(&ctx),
            Layout::Bloco => bloco::render(&ctx),
            Layout::Ma => ma::render(&ctx),
        };
        let grain = if cfg.grain > 0.0 {
            grain_overlay(width, height, cfg.grain)
        } else {
            String::new()
        };

        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" \
             viewBox=\"0 0 {width:.0} {height:.0}\">\
             <rect width=\"{width:.0}\" height=\"{height:.0}\" fill=\"{bg}\"/>\
             {texture}{foreground}{grain}</svg>",
            bg = theme.bg.to_hex(),
        )
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]
    use super::*;
    use crate::cover::format::Format;
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
    fn format_sets_viewbox_dimensions() {
        let mut cfg = CoverConfig::default();
        cfg.format = Format::Social;
        assert!(render_cover_svg(&cfg).contains("viewBox=\"0 0 1200 630\""));
        cfg.format = Format::Portrait;
        assert!(render_cover_svg(&cfg).contains("viewBox=\"0 0 1080 1350\""));
    }

    #[test]
    fn never_emits_japanese_glyphs() {
        // The hard rule: identity comes from geometry/color only.
        let mut cfg = CoverConfig::default();
        for theme in ThemeName::ALL {
            for layout in Layout::ALL {
                for format in Format::ALL {
                    cfg.theme = theme;
                    cfg.layout = layout;
                    cfg.format = format;
                    assert!(!has_cjk(&render_cover_svg(&cfg)), "CJK leaked into the SVG");
                }
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
    fn every_layout_and_format_renders_non_empty_svg() {
        let mut cfg = CoverConfig::default();
        for layout in Layout::ALL {
            for format in Format::ALL {
                cfg.layout = layout;
                cfg.format = format;
                assert!(render_cover_svg(&cfg).len() > 200);
            }
        }
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
