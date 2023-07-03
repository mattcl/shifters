use std::{fs::DirEntry, path::PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use async_std::fs::rename;
use clap::Parser;
use dirs::home_dir;
use filetime::FileTime;
use futures::future::try_join_all;

use crate::config::{Config, PathConfig};

/// A tool for moving files from one set of locations to another.
#[derive(Debug, Clone, Eq, PartialEq, Parser)]
#[command(author, version)]
pub struct Cli {
    /// A path to a config file.
    ///
    /// This defaults to a shift.toml in the user's home directory.
    #[arg(short, long, env = "SHIFTERS_CONFIG")]
    config: Option<PathBuf>,

    /// Actually move files.
    #[arg(short, long)]
    execute: bool,
}

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Self::parse();
        let config_path = if let Some(ref p) = cli.config {
            p.clone()
        } else {
            default_config_path()?
        };
        let config = Config::load(&config_path)?;

        let time = FileTime::now();

        let futures: Vec<_> = config
            .paths
            .iter()
            .map(|(name, conf)| cli.shift(name, conf, time))
            .collect();

        try_join_all(futures).await?;

        Ok(())
    }

    async fn shift(&self, name: &str, conf: &PathConfig, time: FileTime) -> Result<()> {
        if !conf.path.is_dir() {
            bail!(
                "Watched path '{}' for '{}' is not a directory or does not exist",
                conf.path.to_string_lossy(),
                name,
            );
        }

        // make this first, since we're moving them
        let files: Vec<_> = conf
            .path
            .read_dir()?
            .filter_map(|f| {
                f.ok().and_then(|entry| {
                    check_valid_entry(&entry, conf.min_age_seconds.unwrap_or_default(), time)
                        .then_some(entry)
                })
            })
            .collect();

        if files.is_empty() {
            println!("Nothing to do for {}", name);
            return Ok(());
        }

        println!("Shifting {} ({})", name, files.len());

        let futures: Vec<_> = files
            .iter()
            .map(|f| {
                let basename = f.file_name();
                let destination = conf.dest.join(basename);
                self.shift_file(f.path(), destination)
            })
            .collect();

        try_join_all(futures).await?;

        Ok(())
    }

    async fn shift_file(&self, from: PathBuf, to: PathBuf) -> Result<()> {
        if self.execute {
            println!(
                "moving '{}' to '{}'",
                from.to_string_lossy(),
                to.to_string_lossy()
            );
            Ok(rename(&from, to)
                .await
                .with_context(|| format!("failed to move {}", &from.to_string_lossy()))?)
        } else {
            println!(
                "would move '{}' to '{}' but --execute not set",
                from.to_string_lossy(),
                to.to_string_lossy()
            );
            Ok(())
        }
    }
}

fn check_valid_entry(entry: &DirEntry, min_time: u32, time: FileTime) -> bool {
    if !entry.path().is_file() {
        return false;
    }

    if let Ok(meta) = entry.metadata() {
        let modified = FileTime::from_last_modification_time(&meta);
        if modified.seconds() <= time.seconds() - min_time as i64 {
            return true;
        }
    }

    false
}

fn default_config_path() -> Result<PathBuf> {
    Ok(home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))?
        .join("shift.toml"))
}
