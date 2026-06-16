//! Writes covers to disk. Files land in a predictable output directory
//! (`~/Pictures/article-covers`, falling back to home/cwd), named from a slug of
//! the title, never overwriting an existing file.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cover::{CoverConfig, render_cover_svg};
use crate::raster::{EXPORT_PIXELS, png_bytes};

#[derive(Clone, Copy)]
enum Format {
    Svg,
    Png,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Format::Svg => "svg",
            Format::Png => "png",
        }
    }
}

/// Where exported covers are written. Override with `ACAG_OUTPUT_DIR`.
pub fn output_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("ACAG_OUTPUT_DIR") {
        return PathBuf::from(custom);
    }
    dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("article-covers")
}

fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    if out.is_empty() {
        "cover".to_owned()
    } else {
        out
    }
}

fn unique_path(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let mut candidate = dir.join(format!("{stem}.{extension}"));
    let mut counter = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem}-{counter}.{extension}"));
        counter += 1;
    }
    candidate
}

fn write_cover(dir: &Path, config: &CoverConfig, format: Format) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let bytes = match format {
        Format::Svg => render_cover_svg(config).into_bytes(),
        Format::Png => png_bytes(&render_cover_svg(config), EXPORT_PIXELS)?,
    };
    let path = unique_path(dir, &slug(&config.title), format.extension());
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Write the cover as an SVG; returns the path written.
pub fn export_svg(config: &CoverConfig) -> Result<PathBuf> {
    write_cover(&output_dir(), config, Format::Svg)
}

/// Write the cover as a 2160² PNG; returns the path written.
pub fn export_png(config: &CoverConfig) -> Result<PathBuf> {
    write_cover(&output_dir(), config, Format::Png)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("acag-{tag}-{}", std::process::id()))
    }

    #[test]
    fn slug_is_filesystem_safe() {
        assert_eq!(
            slug("Design systems that scale!"),
            "design-systems-that-scale"
        );
        assert_eq!(slug("  Trailing & symbols  "), "trailing-symbols");
        assert_eq!(slug("***"), "cover");
    }

    #[test]
    fn unique_path_avoids_collisions() {
        let dir = temp_dir("unique");
        std::fs::create_dir_all(&dir).unwrap();
        let first = unique_path(&dir, "cover", "png");
        std::fs::write(&first, b"x").unwrap();
        let second = unique_path(&dir, "cover", "png");
        assert_ne!(first, second);
        assert!(second.to_string_lossy().contains("cover-2"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_cover_writes_svg() {
        let dir = temp_dir("svg");
        let config = CoverConfig::default();
        let path = write_cover(&dir, &config, Format::Svg).unwrap();
        assert_eq!(path.extension().unwrap(), "svg");
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.starts_with("<svg"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_cover_writes_png_and_avoids_overwrite() {
        let dir = temp_dir("png");
        let config = CoverConfig::default();
        let first = write_cover(&dir, &config, Format::Png).unwrap();
        let second = write_cover(&dir, &config, Format::Png).unwrap();
        assert_ne!(first, second);
        let bytes = std::fs::read(&first).unwrap();
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_helpers_honor_output_dir_override() {
        let dir = temp_dir("env");
        // SAFETY: single-threaded use of this var; only this test touches it.
        unsafe { std::env::set_var("ACAG_OUTPUT_DIR", &dir) };
        assert_eq!(output_dir(), dir);
        let config = CoverConfig::default();
        let svg = export_svg(&config).unwrap();
        let png = export_png(&config).unwrap();
        assert!(svg.starts_with(&dir) && svg.extension().unwrap() == "svg");
        assert!(png.starts_with(&dir) && png.extension().unwrap() == "png");
        unsafe { std::env::remove_var("ACAG_OUTPUT_DIR") };
        std::fs::remove_dir_all(&dir).ok();
    }
}
