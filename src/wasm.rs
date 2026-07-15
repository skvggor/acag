//! WebAssembly surface for the landing-page hero.
//!
//! Re-exposes the native cover core unchanged to JavaScript: the exact
//! [`render_cover_svg`] the desktop app rasterizes, addressed by flat,
//! index-based parameters, plus a JSON catalog of themes / patterns / layouts /
//! formats so the page builds its controls from the same source of truth. The
//! pure builders are compiled on every target, so their parity with the core is
//! unit-tested natively; only the thin `#[wasm_bindgen]` entry points are
//! wasm-only.

use crate::cover::config::CoverConfig;
use crate::cover::format::Format;
use crate::cover::layouts::Layout;
use crate::cover::render_cover_svg;
use crate::design::patterns::Pattern;
use crate::design::themes::ThemeName;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Build a cover SVG from wasm-friendly flat parameters. Style indices address
/// the `ALL` arrays; out-of-range values fall back to the first variant so a
/// stale page can never panic the module. Sliders are clamped to `[0, 1]`.
#[allow(clippy::too_many_arguments)]
pub fn cover_svg(
    title: &str,
    category: &str,
    date: &str,
    number: &str,
    brand: &str,
    theme: usize,
    pattern: usize,
    layout: usize,
    format: usize,
    pattern_strength: f64,
    grain: f64,
) -> String {
    let config = CoverConfig {
        title: title.to_owned(),
        category: category.to_owned(),
        date: date.to_owned(),
        number: number.to_owned(),
        brand: brand.to_owned(),
        theme: *ThemeName::ALL.get(theme).unwrap_or(&ThemeName::ALL[0]),
        pattern: *Pattern::ALL.get(pattern).unwrap_or(&Pattern::ALL[0]),
        layout: *Layout::ALL.get(layout).unwrap_or(&Layout::ALL[0]),
        format: *Format::ALL.get(format).unwrap_or(&Format::ALL[0]),
        grain: grain.clamp(0.0, 1.0),
        pattern_strength: pattern_strength.clamp(0.0, 1.0),
    };
    render_cover_svg(&config)
}

/// Every theme (with its palette), pattern, layout, and format the app offers,
/// as one JSON document — hand-assembled so the wasm build stays free of a JSON
/// dependency. Labels are romanized ASCII and hex colors, so no escaping is
/// needed.
pub fn catalog_json() -> String {
    let themes: Vec<String> = ThemeName::ALL
        .iter()
        .map(|name| {
            let palette = name.palette();
            format!(
                "{{\"label\":\"{}\",\"bg\":\"{}\",\"line\":\"{}\",\"text\":\"{}\",\"accent\":\"{}\"}}",
                name.label(),
                palette.bg.to_hex(),
                palette.line.to_hex(),
                palette.text.to_hex(),
                palette.accent.to_hex(),
            )
        })
        .collect();
    let patterns: Vec<String> = Pattern::ALL
        .iter()
        .map(|pattern| format!("\"{}\"", pattern.label()))
        .collect();
    let layouts: Vec<String> = Layout::ALL
        .iter()
        .map(|layout| format!("\"{}\"", layout.label()))
        .collect();
    let formats: Vec<String> = Format::ALL
        .iter()
        .map(|format| {
            let (width, height) = format.dimensions();
            format!(
                "{{\"label\":\"{}\",\"width\":{width:.0},\"height\":{height:.0}}}",
                format.label(),
            )
        })
        .collect();
    format!(
        "{{\"themes\":[{}],\"patterns\":[{}],\"layouts\":[{}],\"formats\":[{}]}}",
        themes.join(","),
        patterns.join(","),
        layouts.join(","),
        formats.join(","),
    )
}

/// JavaScript entry point: plate one cover as an SVG string.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn render_cover(
    title: &str,
    category: &str,
    date: &str,
    number: &str,
    brand: &str,
    theme: usize,
    pattern: usize,
    layout: usize,
    format: usize,
    pattern_strength: f64,
    grain: f64,
) -> String {
    cover_svg(
        title,
        category,
        date,
        number,
        brand,
        theme,
        pattern,
        layout,
        format,
        pattern_strength,
        grain,
    )
}

/// JavaScript entry point: the style catalog as JSON.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn catalog() -> String {
    catalog_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_cover(theme: usize, pattern: usize, layout: usize, format: usize) -> String {
        cover_svg(
            "Design systems that scale",
            "engineering",
            "",
            "014",
            "skvggor.dev",
            theme,
            pattern,
            layout,
            format,
            1.0,
            0.0,
        )
    }

    #[test]
    fn renders_the_same_svg_as_the_native_core() {
        let config = CoverConfig {
            title: "Design systems that scale".to_owned(),
            category: "engineering".to_owned(),
            date: String::new(),
            number: "014".to_owned(),
            brand: "skvggor.dev".to_owned(),
            theme: ThemeName::Ai,
            pattern: Pattern::Shippo,
            layout: Layout::Ma,
            format: Format::Social,
            grain: 0.0,
            pattern_strength: 1.0,
        };
        let expected = render_cover_svg(&config);
        let actual = plain_cover(
            ThemeName::Ai.index(),
            Pattern::Shippo.index(),
            Layout::Ma.index(),
            Format::Social.index(),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn format_index_drives_the_viewbox() {
        assert!(plain_cover(0, 0, 0, 0).contains("viewBox=\"0 0 1080 1080\""));
        assert!(plain_cover(0, 0, 0, 1).contains("viewBox=\"0 0 1200 630\""));
        assert!(plain_cover(0, 0, 0, 3).contains("viewBox=\"0 0 1080 1350\""));
    }

    #[test]
    fn out_of_range_indices_fall_back_to_the_first_variant() {
        assert_eq!(plain_cover(99, 99, 99, 99), plain_cover(0, 0, 0, 0));
    }

    #[test]
    fn sliders_are_clamped() {
        let over = cover_svg("T", "", "", "", "", 0, 0, 0, 0, 7.0, 7.0);
        let max = cover_svg("T", "", "", "", "", 0, 0, 0, 0, 1.0, 1.0);
        assert_eq!(over, max);
    }

    #[test]
    fn catalog_lists_every_style_with_palettes() {
        let json = catalog_json();
        for theme in ThemeName::ALL {
            assert!(json.contains(&format!("\"label\":\"{}\"", theme.label())));
            assert!(json.contains(&theme.palette().bg.to_hex()));
        }
        for pattern in Pattern::ALL {
            assert!(json.contains(&format!("\"{}\"", pattern.label())));
        }
        for layout in Layout::ALL {
            assert!(json.contains(&format!("\"{}\"", layout.label())));
        }
        for format in Format::ALL {
            assert!(json.contains(&format!("\"label\":\"{}\"", format.label())));
        }
        assert!(json.contains("\"width\":1200") && json.contains("\"height\":630"));
    }
}
