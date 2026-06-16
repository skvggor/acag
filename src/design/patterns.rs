//! Traditional Japanese patterns (wagara, 和柄) recreated as tileable SVG
//! fragments. Ported from `waka-readme/main.py` so the geometry matches the
//! user's other projects. Each function returns the inner shapes only (paths /
//! circles); the caller wraps them in a `<g stroke=… opacity=…>`.

use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Seigaiha,
    Shippo,
    Kikko,
    Yabane,
    Asanoha,
}

impl Pattern {
    pub const ALL: [Pattern; 5] = [
        Pattern::Seigaiha,
        Pattern::Shippo,
        Pattern::Kikko,
        Pattern::Yabane,
        Pattern::Asanoha,
    ];

    /// Romanized label for the UI only — never drawn on the artwork.
    pub fn label(self) -> &'static str {
        match self {
            Pattern::Seigaiha => "seigaiha",
            Pattern::Shippo => "shippo",
            Pattern::Kikko => "kikko",
            Pattern::Yabane => "yabane",
            Pattern::Asanoha => "asanoha",
        }
    }

    /// Tile size tuned for a 1080² cover (larger = more open spacing).
    pub fn default_scale(self) -> f64 {
        match self {
            Pattern::Seigaiha => 54.0,
            Pattern::Shippo => 60.0,
            Pattern::Kikko => 46.0,
            Pattern::Yabane => 64.0,
            Pattern::Asanoha => 60.0,
        }
    }

    /// Render the pattern shapes covering a `width` × `height` area.
    pub fn shapes(self, width: f64, height: f64, scale: f64) -> String {
        match self {
            Pattern::Seigaiha => seigaiha(width, height, scale),
            Pattern::Shippo => shippo(width, height, scale),
            Pattern::Kikko => kikko(width, height, scale),
            Pattern::Yabane => yabane(width, height, scale),
            Pattern::Asanoha => asanoha(width, height, scale),
        }
    }
}

/// 青海波 — overlapping wave fans of concentric arcs.
fn seigaiha(width: f64, height: f64, scale: f64) -> String {
    let radii = [scale, scale * 2.0 / 3.0, scale / 3.0];
    let mut d = String::new();
    let mut row = 0u32;
    let mut y = 0.0;
    while y <= height + scale {
        let offset = if row % 2 == 1 { scale } else { 0.0 };
        let mut x = -scale + offset;
        while x <= width + scale {
            for r in radii {
                let _ = write!(
                    d,
                    "M{:.1},{:.1} A{:.1},{:.1} 0 0 0 {:.1},{:.1} ",
                    x - r,
                    y,
                    r,
                    r,
                    x + r,
                    y
                );
            }
            x += 2.0 * scale;
        }
        row += 1;
        y += scale;
    }
    format!("<path d=\"{}\"/>", d.trim_end())
}

/// 七宝 — interlocking circles ("seven treasures").
fn shippo(width: f64, height: f64, scale: f64) -> String {
    let mut s = String::new();
    let mut y = 0.0;
    while y <= height + scale {
        let mut x = 0.0;
        while x <= width + scale {
            let _ = write!(s, "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"{scale:.1}\"/>");
            x += scale;
        }
        y += scale;
    }
    s
}

/// 亀甲 — tortoiseshell hexagons.
fn kikko(width: f64, height: f64, scale: f64) -> String {
    let mut s = String::new();
    let hex_w = 3f64.sqrt() * scale;
    let mut row = 0u32;
    let mut cy = 0.0;
    while cy <= height + scale {
        let mut cx = if row % 2 == 1 { hex_w / 2.0 } else { 0.0 };
        while cx <= width + hex_w {
            let mut path = String::from("M");
            for k in 0..6 {
                let angle = (90.0 + 60.0 * f64::from(k)).to_radians();
                let px = cx + scale * angle.cos();
                let py = cy + scale * angle.sin();
                if k == 0 {
                    let _ = write!(path, "{px:.1},{py:.1}");
                } else {
                    let _ = write!(path, " L{px:.1},{py:.1}");
                }
            }
            path.push_str(" Z");
            let _ = write!(s, "<path d=\"{path}\"/>");
            cx += hex_w;
        }
        row += 1;
        cy += 1.5 * scale;
    }
    s
}

/// 矢絣 — arrow-feather chevrons.
fn yabane(width: f64, height: f64, scale: f64) -> String {
    let mut s = String::new();
    let half = scale / 2.0;
    let mut x = 0.0;
    while x <= width + scale {
        let mut y = -scale;
        while y <= height + scale {
            let _ = write!(
                s,
                "<path d=\"M{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1}\"/>",
                x,
                y,
                x + half,
                y + half,
                x + scale,
                y
            );
            let _ = write!(
                s,
                "<path d=\"M{:.1},{:.1} L{:.1},{:.1} L{:.1},{:.1}\"/>",
                x,
                y + half,
                x + half,
                y + scale,
                x + scale,
                y + half
            );
            y += scale;
        }
        x += scale;
    }
    s
}

/// 麻の葉 — hemp-leaf star mesh (triangle medians).
fn asanoha(width: f64, height: f64, scale: f64) -> String {
    let delta_y = scale * 3f64.sqrt() / 2.0;
    let mut d = String::new();

    fn add_medians(d: &mut String, a: (f64, f64), b: (f64, f64), c: (f64, f64)) {
        let mid = |p: (f64, f64), q: (f64, f64)| ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
        for (vertex, opposite) in [(a, mid(b, c)), (b, mid(a, c)), (c, mid(a, b))] {
            let _ = write!(
                d,
                "M{:.1},{:.1} L{:.1},{:.1} ",
                vertex.0, vertex.1, opposite.0, opposite.1
            );
        }
    }

    let offset = |j: i32| {
        if j.rem_euclid(2) != 0 {
            scale / 2.0
        } else {
            0.0
        }
    };
    let rows = (height / delta_y) as i32 + 2;
    let cols = (width / scale) as i32 + 2;
    for j in -1..rows {
        let low = offset(j);
        let high = offset(j + 1);
        for i in -1..cols {
            let (fi, fj) = (f64::from(i), f64::from(j));
            add_medians(
                &mut d,
                (fi * scale + low, fj * delta_y),
                ((fi + 1.0) * scale + low, fj * delta_y),
                ((fi + 0.5) * scale + low, (fj + 1.0) * delta_y),
            );
            add_medians(
                &mut d,
                (fi * scale + high, (fj + 1.0) * delta_y),
                ((fi + 1.0) * scale + high, (fj + 1.0) * delta_y),
                ((fi + 0.5) * scale + high, fj * delta_y),
            );
        }
    }
    format!("<path d=\"{}\"/>", d.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_emits_non_empty_shapes() {
        for pattern in Pattern::ALL {
            let svg = pattern.shapes(200.0, 200.0, pattern.default_scale());
            assert!(
                !svg.is_empty(),
                "pattern {} produced no shapes",
                pattern.label()
            );
            assert!(
                svg.contains("path") || svg.contains("circle"),
                "pattern {} produced unexpected output",
                pattern.label()
            );
        }
    }

    #[test]
    fn shippo_uses_circles_others_use_paths() {
        assert!(
            Pattern::Shippo
                .shapes(100.0, 100.0, 40.0)
                .contains("<circle")
        );
        assert!(
            Pattern::Seigaiha
                .shapes(100.0, 100.0, 40.0)
                .contains("<path")
        );
        assert!(
            Pattern::Asanoha
                .shapes(100.0, 100.0, 40.0)
                .contains("<path")
        );
    }

    #[test]
    fn larger_area_produces_more_geometry() {
        let small = Pattern::Kikko.shapes(100.0, 100.0, 40.0).len();
        let large = Pattern::Kikko.shapes(600.0, 600.0, 40.0).len();
        assert!(large > small);
    }

    #[test]
    fn all_patterns_have_unique_labels() {
        let mut labels: Vec<&str> = Pattern::ALL.iter().map(|p| p.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), Pattern::ALL.len());
    }
}
