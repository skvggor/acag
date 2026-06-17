//! The input model for a cover. Only [`CoverConfig::title`] is required; every
//! other text field is optional (empty = hidden), keeping the artwork generic
//! and usable on any platform.

use serde::{Deserialize, Serialize};

use crate::cover::format::Format;
use crate::cover::layouts::Layout;
use crate::design::patterns::Pattern;
use crate::design::themes::ThemeName;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoverConfig {
    pub title: String,
    pub category: String,
    pub date: String,
    pub number: String,
    pub brand: String,
    pub theme: ThemeName,
    pub pattern: Pattern,
    pub layout: Layout,
    pub format: Format,
    /// Film-grain intensity in `[0, 1]` (0 = off).
    pub grain: f64,
    /// Multiplier in `[0, 1]` for the wagara texture and seal opacity (0 = hidden).
    pub pattern_strength: f64,
}

impl Default for CoverConfig {
    fn default() -> Self {
        Self {
            title: "Design systems that scale".to_owned(),
            category: "Engineering".to_owned(),
            date: "16 Jun 2026".to_owned(),
            number: "014".to_owned(),
            brand: "skvggor.dev".to_owned(),
            theme: ThemeName::Terracotta,
            pattern: Pattern::Seigaiha,
            layout: Layout::Editorial,
            format: Format::Square,
            grain: 0.0,
            pattern_strength: 1.0,
        }
    }
}

impl CoverConfig {
    /// "Omakase" — randomize only the visual style (theme, pattern, layout,
    /// grain), keeping all of the user's text. Trusts the house to plate it.
    pub fn randomize_style(&mut self) {
        self.theme = *fastrand::choice(ThemeName::ALL.iter()).expect("non-empty");
        self.pattern = *fastrand::choice(Pattern::ALL.iter()).expect("non-empty");
        self.layout = *fastrand::choice(Layout::ALL.iter()).expect("non-empty");
        self.grain = if fastrand::bool() {
            0.18 + fastrand::f64() * 0.22
        } else {
            0.0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn randomize_keeps_text_and_varies_style() {
        fastrand::seed(42);
        let mut config = CoverConfig::default();
        let title = config.title.clone();
        let mut layouts = std::collections::HashSet::new();
        for _ in 0..40 {
            config.randomize_style();
            assert_eq!(config.title, title, "text must be preserved");
            layouts.insert(config.layout);
        }
        // Over many draws every layout should appear.
        assert_eq!(layouts.len(), Layout::ALL.len());
    }
}
