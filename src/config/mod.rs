use anyhow::{Context, Result};
use serde::Deserialize;

/// Top-level starmap.toml schema.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default)]
    pub llms_full: LlmsFullConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmsFullConfig {
    #[serde(default = "default_max_readme_size_kb")]
    pub max_readme_size_kb: usize,
}

impl Default for LlmsFullConfig {
    fn default() -> Self {
        Self {
            max_readme_size_kb: default_max_readme_size_kb(),
        }
    }
}

fn default_max_readme_size_kb() -> usize {
    10
}

impl Config {
    /// Parse a TOML string into a Config.
    pub fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s).context("Invalid starmap.toml")
    }

    /// Load starmap.toml from the given directory (no traversal upward).
    /// Returns Config::default() if the file does not exist.
    pub fn load_from_dir(dir: &std::path::Path) -> Result<Self> {
        let path = dir.join("starmap.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Self::from_str(&contents)
    }

    /// Convenience: load from the current working directory.
    pub fn load() -> Result<Self> {
        let cwd = std::env::current_dir().context("Failed to read current dir")?;
        Self::load_from_dir(&cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml = r#"
order = ["🤖 AI", "🎉 Other"]

[llms_full]
max_readme_size_kb = 20
"#;
        let cfg = Config::from_str(toml).unwrap();
        assert_eq!(cfg.order, vec!["🤖 AI", "🎉 Other"]);
        assert_eq!(cfg.llms_full.max_readme_size_kb, 20);
    }

    #[test]
    fn parse_order_only() {
        let cfg = Config::from_str(r#"order = ["a", "b"]"#).unwrap();
        assert_eq!(cfg.order, vec!["a", "b"]);
        assert_eq!(cfg.llms_full.max_readme_size_kb, 10);
    }

    #[test]
    fn parse_empty() {
        let cfg = Config::from_str("").unwrap();
        assert!(cfg.order.is_empty());
        assert_eq!(cfg.llms_full.max_readme_size_kb, 10);
    }

    #[test]
    fn unknown_field_errors() {
        let result = Config::from_str(r#"unknown = "x""#);
        assert!(result.is_err());
    }

    #[test]
    fn load_from_dir_finds_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("starmap.toml"), r#"order = ["X"]"#).unwrap();
        let cfg = Config::load_from_dir(dir.path()).unwrap();
        assert_eq!(cfg.order, vec!["X"]);
    }

    #[test]
    fn load_from_dir_returns_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config::load_from_dir(dir.path()).unwrap();
        assert!(cfg.order.is_empty());
    }
}
