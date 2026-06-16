//! Render the sample covers used in the README to `docs/samples`.
//! Run with: `cargo run --example gallery`.

use article_cover_art_generator::cover::config::CoverConfig;
use article_cover_art_generator::cover::layouts::Layout;
use article_cover_art_generator::cover::render_cover_svg;
use article_cover_art_generator::design::patterns::Pattern;
use article_cover_art_generator::design::themes::ThemeName;
use article_cover_art_generator::raster::png_bytes;

fn main() -> anyhow::Result<()> {
    let out = std::path::Path::new("docs/samples");
    std::fs::create_dir_all(out)?;

    let samples = [
        (
            ThemeName::Terracotta,
            Layout::Editorial,
            Pattern::Seigaiha,
            "Design systems that scale",
        ),
        (
            ThemeName::Sumi,
            Layout::Bloco,
            Pattern::Asanoha,
            "Performance without the magic",
        ),
        (
            ThemeName::Ai,
            Layout::Ma,
            Pattern::Shippo,
            "The quiet art of refactoring legacy code",
        ),
        (
            ThemeName::Matcha,
            Layout::Editorial,
            Pattern::Kikko,
            "Type-driven development",
        ),
        (
            ThemeName::Sakura,
            Layout::Ma,
            Pattern::Yabane,
            "Writing tests you will not regret",
        ),
    ];

    for (index, (theme, layout, pattern, title)) in samples.iter().enumerate() {
        let config = CoverConfig {
            title: (*title).to_owned(),
            number: format!("{:03}", index + 1),
            theme: *theme,
            layout: *layout,
            pattern: *pattern,
            ..Default::default()
        };
        let svg = render_cover_svg(&config);
        let name = format!("{index}-{}-{}", theme.label(), layout.label());
        std::fs::write(out.join(format!("{name}.png")), png_bytes(&svg, 1080)?)?;
        println!("wrote {name}");
    }

    println!("gallery at {}", out.display());
    Ok(())
}
