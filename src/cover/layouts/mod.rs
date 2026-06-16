//! The three cover composition presets. Each submodule exposes
//! `render(ctx) -> String` returning the foreground SVG (text, seal, accents);
//! the background and texture are added by [`crate::cover::render`].

pub mod bloco;
pub mod editorial;
pub mod ma;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    Editorial,
    Bloco,
    Ma,
}

impl Layout {
    pub const ALL: [Layout; 3] = [Layout::Editorial, Layout::Bloco, Layout::Ma];

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
}
