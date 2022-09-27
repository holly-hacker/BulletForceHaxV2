mod downloader_dialog;
mod scraper;

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use self::downloader_dialog::DownloaderDialog;

const CONFIG_FILE: &str = "version.json";

pub struct VersionManager {
    path: PathBuf,
}

impl VersionManager {
    pub fn new(base_path: &Path) -> Result<Self> {
        std::fs::create_dir_all(base_path)
            .with_context(|| format!("create directories for config: {base_path:?}"))?;

        Ok(Self {
            path: base_path.to_owned(),
        })
    }

    pub fn version(&self) -> Option<VersionConfig> {
        let config_path = self.path.join(CONFIG_FILE);
        match config_path.try_exists() {
            Ok(true) => VersionConfig::read_from_path(&config_path).ok(),
            _ => None,
        }
    }

    pub fn show_version_downloader_blocking(&self) -> Result<Option<VersionConfig>> {
        let downloaded_files = DownloaderDialog::show_dialog().ok();

        let mut loader = None;
        let mut json = None;

        if let Some(downloaded_files) = downloaded_files {
            for file in downloaded_files {
                info!("Downloaded file {}", file.name);
                let path = self.path.join(&file.name);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, file.data)?;

                match file.file_type {
                    scraper::FileType::UnityLoader => loader = Some(file.name),
                    scraper::FileType::GameJson => json = Some(file.name),
                    scraper::FileType::GameFile => (),
                }
            }

            let config = VersionConfig {
                base_path: self.path.clone(),
                unity_loader: loader.ok_or_else(|| anyhow!("did not find unity loader"))?,
                game_json: json.ok_or_else(|| anyhow!("did not find game json"))?,
            };

            config.write_to_file(&self.path.join(CONFIG_FILE))?;

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}

/// The config file that gets written do disk inside the config directory
#[derive(Default, Serialize, Deserialize)]
pub struct VersionConfig {
    base_path: PathBuf,
    unity_loader: String,
    game_json: String,
}

impl VersionConfig {
    pub fn get_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    pub fn get_game_json(&self) -> PathBuf {
        self.base_path.join(&self.game_json)
    }

    pub fn get_unity_loader(&self) -> PathBuf {
        self.base_path.join(&self.unity_loader)
    }

    fn read_from_path(path: &Path) -> Result<Self> {
        let read = std::fs::read(path)?;
        let read = serde_json::from_slice(&read)?;
        Ok(read)
    }

    fn write_to_file(&self, path: &Path) -> Result<()> {
        let to_write = serde_json::to_vec_pretty(self)?;
        std::fs::write(path, to_write)?;
        Ok(())
    }
}
