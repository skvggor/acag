//! Named presets: save the current look under a name and load any saved one
//! later. Each preset is a TOML file in the presets directory
//! (`~/.config/article-cover-art-generator/presets/`, override with
//! `ACAG_PRESETS_DIR`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cover::CoverConfig;
use crate::export::slug;

/// Directory holding the preset files.
pub fn presets_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("ACAG_PRESETS_DIR") {
        return PathBuf::from(custom);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("article-cover-art-generator")
        .join("presets")
}

fn list_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "toml"))
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_owned)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

fn save_in(dir: &Path, name: &str, config: &CoverConfig) -> Result<String> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let stem = slug(name);
    let path = dir.join(format!("{stem}.toml"));
    let toml = toml::to_string_pretty(config).context("serializing preset")?;
    std::fs::write(&path, toml).with_context(|| format!("writing {}", path.display()))?;
    Ok(stem)
}

fn load_in(dir: &Path, name: &str) -> Result<CoverConfig> {
    let path = dir.join(format!("{name}.toml"));
    let toml =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&toml).context("parsing preset")
}

fn delete_in(dir: &Path, name: &str) -> Result<()> {
    let path = dir.join(format!("{name}.toml"));
    std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    Ok(())
}

/// Names of all saved presets, sorted.
pub fn list() -> Vec<String> {
    list_in(&presets_dir())
}

/// Save `config` under `name`; returns the stored (slugged) name.
pub fn save(name: &str, config: &CoverConfig) -> Result<String> {
    save_in(&presets_dir(), name, config)
}

/// Load the preset stored under `name`.
pub fn load(name: &str) -> Result<CoverConfig> {
    load_in(&presets_dir(), name)
}

/// Delete the preset stored under `name`.
pub fn delete(name: &str) -> Result<()> {
    delete_in(&presets_dir(), name)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]
    use super::*;
    use crate::design::themes::ThemeName;

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("acag-presets-{tag}-{}", std::process::id()))
    }

    #[test]
    fn save_list_load_and_delete_roundtrip() {
        let dir = temp_dir("roundtrip");
        std::fs::remove_dir_all(&dir).ok();

        assert!(list_in(&dir).is_empty());

        let mut config = CoverConfig::default();
        config.theme = ThemeName::Ai;
        let stored = save_in(&dir, "My LinkedIn Look!", &config).unwrap();
        assert_eq!(stored, "my-linkedin-look");

        assert_eq!(list_in(&dir), vec!["my-linkedin-look".to_owned()]);

        let loaded = load_in(&dir, &stored).unwrap();
        assert_eq!(loaded.theme, ThemeName::Ai);

        delete_in(&dir, &stored).unwrap();
        assert!(list_in(&dir).is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_sorts_and_ignores_non_toml() {
        let dir = temp_dir("sort");
        std::fs::create_dir_all(&dir).unwrap();
        save_in(&dir, "zeta", &CoverConfig::default()).unwrap();
        save_in(&dir, "alpha", &CoverConfig::default()).unwrap();
        std::fs::write(dir.join("notes.txt"), b"ignore me").unwrap();
        assert_eq!(list_in(&dir), vec!["alpha".to_owned(), "zeta".to_owned()]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_and_delete_error_on_missing() {
        let dir = temp_dir("missing");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_in(&dir, "nope").is_err());
        assert!(delete_in(&dir, "nope").is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn presets_dir_honors_env_override() {
        // SAFETY: only this test touches this variable.
        unsafe { std::env::set_var("ACAG_PRESETS_DIR", "/tmp/acag-presets-x") };
        assert_eq!(presets_dir(), PathBuf::from("/tmp/acag-presets-x"));
        unsafe { std::env::remove_var("ACAG_PRESETS_DIR") };
    }
}
