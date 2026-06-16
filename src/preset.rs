//! Save and load a cover preset as TOML, to reuse a look across articles.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cover::CoverConfig;

/// Default preset location: `~/.config/article-cover-art-generator/preset.toml`.
pub fn default_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("article-cover-art-generator")
        .join("preset.toml")
}

pub fn save(config: &CoverConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let toml = toml::to_string_pretty(config).context("serializing preset")?;
    std::fs::write(path, toml).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<CoverConfig> {
    let toml =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&toml).context("parsing preset")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]
    use super::*;
    use crate::design::themes::ThemeName;

    #[test]
    fn roundtrips_through_toml() {
        let mut config = CoverConfig::default();
        config.theme = ThemeName::Ai;
        config.grain = 0.3;
        config.title = "Reusable look".to_owned();
        let toml = toml::to_string_pretty(&config).unwrap();
        let back: CoverConfig = toml::from_str(&toml).unwrap();
        assert_eq!(back.theme, ThemeName::Ai);
        assert_eq!(back.title, "Reusable look");
        assert!((back.grain - 0.3).abs() < 1e-9);
    }

    #[test]
    fn saves_and_loads_a_file() {
        let path = std::env::temp_dir().join(format!("acag-preset-{}.toml", std::process::id()));
        let config = CoverConfig::default();
        save(&config, &path).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.title, config.title);
        assert_eq!(loaded.pattern, config.pattern);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn default_path_points_at_config_dir() {
        let path = default_path();
        assert!(path.ends_with("preset.toml"));
        assert!(
            path.to_string_lossy()
                .contains("article-cover-art-generator")
        );
    }

    #[test]
    fn load_errors_on_missing_and_invalid_files() {
        assert!(load(Path::new("/no/such/acag/preset.toml")).is_err());
        let path = std::env::temp_dir().join(format!("acag-bad-{}.toml", std::process::id()));
        std::fs::write(&path, "this = is = not = toml").unwrap();
        assert!(load(&path).is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_errors_when_parent_is_unusable() {
        let blocker = std::env::temp_dir().join(format!("acag-pblock-{}", std::process::id()));
        std::fs::write(&blocker, b"x").unwrap();
        let path = blocker.join("preset.toml");
        assert!(save(&CoverConfig::default(), &path).is_err());
        std::fs::remove_file(&blocker).ok();
    }
}
