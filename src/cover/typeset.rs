//! Text measurement against the bundled Montserrat faces, so titles can be
//! wrapped and auto-sized precisely instead of guessed. Uses real glyph advance
//! widths via `ttf-parser`.

use ttf_parser::Face;

/// Baseline-to-baseline factor used both when fitting a title and when stacking
/// its lines, so the two stay in sync.
pub const LINE_HEIGHT: f64 = 1.06;

#[derive(Clone, Copy, Debug)]
pub enum Weight {
    Regular,
    Bold,
    Black,
}

impl Weight {
    /// CSS/SVG `font-weight` value.
    pub fn css(self) -> u16 {
        match self {
            Weight::Regular => 400,
            Weight::Bold => 700,
            Weight::Black => 900,
        }
    }
}

fn bytes(weight: Weight) -> &'static [u8] {
    match weight {
        Weight::Regular => include_bytes!("../../assets/fonts/Montserrat-Regular.ttf"),
        Weight::Bold => include_bytes!("../../assets/fonts/Montserrat-Bold.ttf"),
        Weight::Black => include_bytes!("../../assets/fonts/Montserrat-Black.ttf"),
    }
}

/// A parsed Montserrat face used only for layout math (not rendering).
pub struct Font {
    face: Face<'static>,
}

impl Font {
    pub fn new(weight: Weight) -> Self {
        Self {
            face: Face::parse(bytes(weight), 0).expect("bundled Montserrat face is valid"),
        }
    }

    /// Advance width of `text` at `font_size`, in user units.
    pub fn text_width(&self, text: &str, font_size: f64) -> f64 {
        let upem = f64::from(self.face.units_per_em());
        text.chars()
            .map(|ch| {
                let advance = self
                    .face
                    .glyph_index(ch)
                    .and_then(|g| self.face.glyph_hor_advance(g))
                    .unwrap_or(0);
                f64::from(advance) / upem * font_size
            })
            .sum()
    }

    /// Greedy word wrap to `max_width`. Never drops words; an over-long single
    /// word simply overflows its line.
    pub fn wrap(&self, text: &str, font_size: f64, max_width: f64) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in text.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if current.is_empty() || self.text_width(&candidate, font_size) <= max_width {
                current = candidate;
            } else {
                lines.push(std::mem::take(&mut current));
                current = word.to_owned();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    /// Largest size (stepping down from `max_size`) whose wrapped text fits both
    /// `max_width` and `max_height`. Returns the chosen size and wrapped lines so
    /// the title adapts to any aspect ratio (fewer/smaller lines when short).
    pub fn fit_box(
        &self,
        text: &str,
        max_width: f64,
        max_height: f64,
        max_size: f64,
        min_size: f64,
    ) -> (f64, Vec<String>) {
        let mut size = max_size;
        while size > min_size {
            let lines = self.wrap(text, size, max_width);
            let total_height = lines.len() as f64 * size * LINE_HEIGHT;
            let fits_width = lines.iter().all(|l| self.text_width(l, size) <= max_width);
            if fits_width && total_height <= max_height {
                return (size, lines);
            }
            size -= 2.0;
        }
        (min_size, self.wrap(text, min_size, max_width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_positive_width() {
        let font = Font::new(Weight::Black);
        let width = font.text_width("Design", 100.0);
        assert!(width > 0.0);
        // A longer string is wider.
        assert!(font.text_width("Design systems", 100.0) > width);
    }

    #[test]
    fn wrap_splits_long_text() {
        let font = Font::new(Weight::Black);
        let lines = font.wrap("one two three four five six", 80.0, 300.0);
        assert!(lines.len() > 1);
    }

    #[test]
    fn fit_box_keeps_text_within_bounds() {
        let font = Font::new(Weight::Black);
        let (max_w, max_h) = (800.0, 360.0);
        let (size, lines) = font.fit_box("Design systems that scale", max_w, max_h, 150.0, 40.0);
        assert!((40.0..=150.0).contains(&size));
        assert!(
            lines
                .iter()
                .all(|line| font.text_width(line, size) <= max_w + 1.0)
        );
        // Height fits unless it had to fall back to the minimum size.
        if size > 40.0 {
            assert!(lines.len() as f64 * size * LINE_HEIGHT <= max_h + 0.5);
        }
    }

    #[test]
    fn shorter_box_picks_a_smaller_or_equal_size() {
        let font = Font::new(Weight::Black);
        let text = "The quiet art of refactoring legacy code";
        let (tall, _) = font.fit_box(text, 900.0, 700.0, 150.0, 30.0);
        let (short, _) = font.fit_box(text, 900.0, 240.0, 150.0, 30.0);
        assert!(short <= tall);
    }
}
