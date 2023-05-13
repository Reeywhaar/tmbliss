use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Conf {
    pub paths: Vec<String>,

    #[serde(default)]
    pub whitelist_glob: Vec<String>,

    #[serde(default)]
    pub whitelist_path: Vec<String>,

    #[serde(default)]
    pub skip_glob: Vec<String>,

    #[serde(default)]
    pub skip_path: Vec<String>,

    #[serde(default)]
    pub dry_run: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            whitelist_glob: Vec::new(),
            whitelist_path: Vec::new(),
            skip_glob: Vec::new(),
            skip_path: Vec::new(),
            dry_run: true,
        }
    }
}

impl Conf {
    pub fn parse(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .with_context(|| format!("Cannot create reader for path {}", path))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_parses_config() {
        let conf = super::Conf::parse("./test_assets/test_config.json").unwrap();

        assert_eq!(conf.paths, ["./test_assets/test_dir"]);
        assert_eq!(conf.whitelist_glob.len(), 2);
        assert_eq!(conf.whitelist_glob, ["**/.env", "**/.env.*"]);
        assert!(conf.dry_run);
    }

    #[test]
    fn it_fails_if_no_paths_provided() {
        let conf = super::Conf::parse("./test_assets/test_config_no_paths.json");

        assert!(conf.is_err());
    }

    #[test]
    fn it_parses_config_with_missing_excludes() {
        let conf = super::Conf::parse("./test_assets/test_config_no_exclude.json").unwrap();

        assert_eq!(conf.paths, ["./test_assets/test_dir"]);
        assert_eq!(conf.whitelist_glob.len(), 0);
        assert!(conf.dry_run);
    }
}