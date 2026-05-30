#![allow(dead_code)]

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
}
