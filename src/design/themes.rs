//! The six sumi-ê color themes, shared with `personal-website` and `waka-readme`.
//! Each one is a "poster" variant (saturated background) chosen so that the main
//! text already clears WCAG AAA (7:1) against the background.

use serde::{Deserialize, Serialize};

use crate::design::contrast::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeName {
    Terracotta,
    Sumi,
    Matcha,
    Washi,
    Ai,
    Sakura,
}

impl ThemeName {
    pub const ALL: [ThemeName; 6] = [
        ThemeName::Terracotta,
        ThemeName::Sumi,
        ThemeName::Matcha,
        ThemeName::Washi,
        ThemeName::Ai,
        ThemeName::Sakura,
    ];

    /// Position in [`ThemeName::ALL`], for syncing with UI combo boxes.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&item| item == self).unwrap_or(0)
    }

    /// Romanized label for the UI only — never drawn on the artwork.
    pub fn label(self) -> &'static str {
        match self {
            ThemeName::Terracotta => "terracotta",
            ThemeName::Sumi => "sumi",
            ThemeName::Matcha => "matcha",
            ThemeName::Washi => "washi",
            ThemeName::Ai => "ai",
            ThemeName::Sakura => "sakura",
        }
    }

    pub fn palette(self) -> Theme {
        // (background, ink line, text, accent). Values mirror the `--poster-*`
        // tokens in personal-website; the accent is the theme's signature hue.
        let (bg, line, text, accent) = match self {
            ThemeName::Terracotta => ("#d89868", "#2a1810", "#1e1108", "#9a4a24"),
            ThemeName::Sumi => ("#1c1a17", "#d4c4a8", "#d4c4a8", "#c0703c"),
            ThemeName::Matcha => ("#354028", "#d8e0c8", "#d8e0c8", "#9fb074"),
            ThemeName::Washi => ("#d8cab0", "#2a1810", "#2a1810", "#8b5a2c"),
            ThemeName::Ai => ("#1b3a4b", "#c8d8e4", "#c8d8e4", "#5b93b0"),
            ThemeName::Sakura => ("#deb0b0", "#2e1a1a", "#2e1a1a", "#9a4a4a"),
        };
        Theme {
            name: self,
            bg: Rgb::from_hex(bg),
            line: Rgb::from_hex(line),
            text: Rgb::from_hex(text),
            accent: Rgb::from_hex(accent),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub name: ThemeName,
    /// Background fill of the whole cover.
    pub bg: Rgb,
    /// Ink color for wagara strokes and dividers — contrasts with `bg`.
    pub line: Rgb,
    /// Main text color; clears AAA against `bg` by construction.
    pub text: Rgb,
    /// Chromatic signature hue (kicker tick, seal highlights).
    pub accent: Rgb,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::contrast::contrast_ratio;

    #[test]
    fn every_theme_text_clears_aaa() {
        for name in ThemeName::ALL {
            let theme = name.palette();
            let ratio = contrast_ratio(theme.text, theme.bg);
            assert!(
                ratio >= 7.0,
                "theme {} text/bg contrast {ratio:.2} < 7:1",
                name.label()
            );
        }
    }

    #[test]
    fn every_theme_line_is_visible_over_bg() {
        // The ink line is used for the wagara seal; it must stand out clearly.
        for name in ThemeName::ALL {
            let theme = name.palette();
            assert!(
                contrast_ratio(theme.line, theme.bg) >= 4.5,
                "theme {} line/bg contrast too low",
                name.label()
            );
        }
    }

    #[test]
    fn all_themes_have_unique_labels() {
        let mut labels: Vec<&str> = ThemeName::ALL.iter().map(|t| t.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), ThemeName::ALL.len());
    }

    #[test]
    fn index_matches_position() {
        for (position, theme) in ThemeName::ALL.iter().enumerate() {
            assert_eq!(theme.index(), position);
        }
    }
}
