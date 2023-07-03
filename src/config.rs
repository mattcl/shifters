use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Deserialize)]
pub struct Global {
    #[serde(default)]
    pub min_age_seconds: u32,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
pub struct PathConfig {
    pub path: PathBuf,
    pub dest: PathBuf,
    #[serde(default)]
    pub min_age_seconds: Option<u32>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
pub struct Config {
    pub global: Global,
    pub paths: HashMap<String, PathConfig>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let mut config: Config = Figment::new().merge(Toml::file(path)).extract()?;

        for (_, val) in config.paths.iter_mut() {
            if config.global.min_age_seconds > 0 && val.min_age_seconds.is_none() {
                val.min_age_seconds = Some(config.global.min_age_seconds);
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_format() {
        let input = r#"[global]
min_age_seconds = 500

[paths.foo]
path = "/bar/foo"
dest = "/foo/bar"
        "#;

        let expected_global = Global {
            min_age_seconds: 500,
        };

        let expected_path = PathConfig {
            path: PathBuf::from("/bar/foo"),
            dest: PathBuf::from("/foo/bar"),
            min_age_seconds: None,
        };

        let config: Config = Figment::new().merge(Toml::string(input)).extract().unwrap();

        assert_eq!(config.global, expected_global);
        assert_eq!(config.paths.get("foo").unwrap(), &expected_path);
    }
}
