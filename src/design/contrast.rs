//! Color helpers and WCAG contrast utilities.
//!
//! The accessibility rule for this project is strict: any readable text must
//! reach a contrast ratio of at least 7:1 (WCAG AAA) against its immediate
//! background. [`ensure_readable`] is the guardrail used across the renderer.

/// An sRGB color in 8-bit channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
    pub const WHITE: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a `#rrggbb` (or `rrggbb`) hex string. Falls back to black on
    /// malformed input so callers never have to handle an error for static
    /// palette literals.
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Self::BLACK;
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
        Self::new(channel(0), channel(2), channel(4))
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Linear blend toward `other` by `ratio` in `[0, 1]`.
    pub fn mix(self, other: Rgb, ratio: f64) -> Rgb {
        let ratio = ratio.clamp(0.0, 1.0);
        let lerp =
            |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * ratio).round() as u8;
        Rgb::new(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
        )
    }

    /// WCAG relative luminance in `[0, 1]`.
    pub fn relative_luminance(self) -> f64 {
        fn linear(c: u8) -> f64 {
            let c = f64::from(c) / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }
}

/// WCAG contrast ratio between two colors. Ranges from 1.0 (identical) to 21.0
/// (black vs. white).
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (a.relative_luminance(), b.relative_luminance());
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Darken or lighten `fg` — toward black or white, whichever raises contrast
/// with `bg` — until it meets `target`, preserving as much of the original hue
/// as possible. Mixing toward the anchor is monotonic in contrast, so when the
/// target is unreachable this returns the extreme that maximizes it.
pub fn ensure_readable(fg: Rgb, bg: Rgb, target: f64) -> Rgb {
    if contrast_ratio(fg, bg) >= target {
        return fg;
    }
    // Push the text toward whichever extreme contrasts most with the background.
    // A fixed luminance threshold misjudges mid-luminance hues (e.g. a peach
    // background that looks light but sits below 0.5), so compare directly.
    let anchor = if contrast_ratio(Rgb::BLACK, bg) >= contrast_ratio(Rgb::WHITE, bg) {
        Rgb::BLACK
    } else {
        Rgb::WHITE
    };
    const STEPS: u32 = 24;
    for step in 1..=STEPS {
        let candidate = fg.mix(anchor, f64::from(step) / f64::from(STEPS));
        if contrast_ratio(candidate, bg) >= target {
            return candidate;
        }
    }
    anchor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_hex() {
        let c = Rgb::from_hex("#d89868");
        assert_eq!(c, Rgb::new(0xd8, 0x98, 0x68));
        assert_eq!(c.to_hex(), "#d89868");
        // Tolerates a missing leading '#'.
        assert_eq!(Rgb::from_hex("1e1108"), Rgb::new(0x1e, 0x11, 0x08));
    }

    #[test]
    fn black_white_ratio_is_21() {
        let ratio = contrast_ratio(Rgb::BLACK, Rgb::WHITE);
        assert!((ratio - 21.0).abs() < 0.01, "expected ~21, got {ratio}");
        // Order independent.
        assert_eq!(contrast_ratio(Rgb::WHITE, Rgb::BLACK), ratio);
    }

    #[test]
    fn identical_colors_ratio_is_1() {
        assert!((contrast_ratio(Rgb::BLACK, Rgb::BLACK) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mix_endpoints_and_midpoint() {
        let a = Rgb::new(0, 0, 0);
        let b = Rgb::new(100, 200, 50);
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
        assert_eq!(a.mix(b, 0.5), Rgb::new(50, 100, 25));
    }

    #[test]
    fn ensure_readable_reaches_target_on_low_contrast_pair() {
        // Two mid tones with poor contrast.
        let fg = Rgb::from_hex("#b87040");
        let bg = Rgb::from_hex("#d89868");
        assert!(contrast_ratio(fg, bg) < 7.0);
        let fixed = ensure_readable(fg, bg, 7.0);
        assert!(
            contrast_ratio(fixed, bg) >= 7.0,
            "ratio after fix was {}",
            contrast_ratio(fixed, bg)
        );
    }

    #[test]
    fn ensure_readable_is_identity_when_already_passing() {
        let fg = Rgb::BLACK;
        let bg = Rgb::WHITE;
        assert_eq!(ensure_readable(fg, bg, 7.0), fg);
    }
}
