//! The three cover composition presets. Each submodule exposes
//! `render(ctx) -> String` returning the foreground SVG (text, seal, accents);
//! the background and texture are added by [`crate::cover::render`].

pub mod bloco;
pub mod editorial;
pub mod ma;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layout {
    Editorial,
    Bloco,
    Ma,
}

impl Layout {
    pub const ALL: [Layout; 3] = [Layout::Editorial, Layout::Bloco, Layout::Ma];

    /// Position in [`Layout::ALL`], for syncing with UI combo boxes.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&item| item == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            Layout::Editorial => "editorial",
            Layout::Bloco => "bloco",
            Layout::Ma => "ma",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_unique_and_cover_all_variants() {
        let labels: Vec<&str> = Layout::ALL.iter().map(|l| l.label()).collect();
        assert_eq!(labels, ["editorial", "bloco", "ma"]);
    }

    #[test]
    fn index_matches_position() {
        for (position, layout) in Layout::ALL.iter().enumerate() {
            assert_eq!(layout.index(), position);
        }
    }
}
