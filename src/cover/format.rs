//! Output formats (aspect ratios). The design canvas is sized per format and the
//! layouts compose against its width/height, so a cover works as a square, a
//! social/link card, a wide banner, or a portrait.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    /// 1:1 — Instagram feed, generic.
    Square,
    /// 1.91:1 — Open Graph link previews (LinkedIn, Facebook, X, Slack…).
    Social,
    /// 16:9 — native LinkedIn article cover, YouTube, slides, blog headers.
    Wide,
    /// 4:5 — Instagram portrait, vertical feeds.
    Portrait,
    /// 2:1 — wide banners (dev.to, X 2:1 cards).
    Banner,
}

impl Format {
    pub const ALL: [Format; 5] = [
        Format::Square,
        Format::Social,
        Format::Wide,
        Format::Portrait,
        Format::Banner,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&item| item == self).unwrap_or(0)
    }

    /// Label for the UI combo box.
    pub fn label(self) -> &'static str {
        match self {
            Format::Square => "1:1 · square",
            Format::Social => "1.91:1 · link",
            Format::Wide => "16:9 · wide",
            Format::Portrait => "4:5 · portrait",
            Format::Banner => "2:1 · banner",
        }
    }

    /// Design canvas (user units) the layouts compose in; also the SVG viewBox.
    pub fn dimensions(self) -> (f64, f64) {
        match self {
            Format::Square => (1080.0, 1080.0),
            Format::Social => (1200.0, 630.0),
            Format::Wide => (1920.0, 1080.0),
            Format::Portrait => (1080.0, 1350.0),
            Format::Banner => (1200.0, 600.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_matches_position_and_labels_unique() {
        for (position, format) in Format::ALL.iter().enumerate() {
            assert_eq!(format.index(), position);
        }
        let mut labels: Vec<&str> = Format::ALL.iter().map(|f| f.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), Format::ALL.len());
    }

    #[test]
    fn dimensions_have_expected_orientation() {
        assert_eq!(Format::Square.dimensions(), (1080.0, 1080.0));
        // Wide formats are landscape, portrait is taller than wide.
        let (w, h) = Format::Social.dimensions();
        assert!(w > h);
        let (w, h) = Format::Portrait.dimensions();
        assert!(h > w);
    }
}
