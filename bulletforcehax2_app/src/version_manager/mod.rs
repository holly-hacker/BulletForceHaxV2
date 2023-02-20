mod downloader_dialog;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bulletforcehax2_lib::version_scraper::FileType;
use serde::{Deserialize, Serialize};
use tracing::info;

use self::downloader_dialog::DownloaderDialog;

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
        match self.path.try_exists() {
            Ok(true) => VersionConfig::read_from_directory(&self.path).ok(),
            _ => None,
        }
    }

    pub fn show_version_downloader_blocking(&self) -> Result<Option<VersionConfig>> {
        let downloaded_files = DownloaderDialog::show_dialog().ok();

        let mut loader = None;
        let mut code = None;
        let mut data = None;
        let mut framework = None;

        if let Some(downloaded_files) = downloaded_files {
            for file in downloaded_files {
                info!("Downloaded file {}", file.name);
                let path = self.path.join(&file.name);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, file.data)?;

                match file.file_type {
                    FileType::UnityLoader => loader = Some(file.name),
                    FileType::Framework => framework = Some(file.name),
                    FileType::Code => code = Some(file.name),
                    FileType::Data => data = Some(file.name),
                }
            }

            let config = VersionConfig {
                base_path: self.path.clone(),
                unity_loader: loader.context("did not find unity loader")?,
                code: code.context("did not find code file")?,
                data: data.context("did not find data file")?,
                framework: framework.context("did not find framework file")?,
            };

            config.write_to_directory(&self.path)?;

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}

/// The config file that gets written do disk inside the config directory
#[derive(Default, Serialize, Deserialize)]
pub struct VersionConfig {
    #[serde(skip)]
    base_path: PathBuf,
    unity_loader: String,
    code: String,
    data: String,
    framework: String,
}

impl VersionConfig {
    const CONFIG_FILE: &'static str = "version.json";

    pub fn get_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    pub fn get_unity_loader(&self) -> PathBuf {
        self.base_path.join(&self.unity_loader)
    }

    pub fn get_code(&self) -> PathBuf {
        self.base_path.join(&self.code)
    }

    pub fn get_data(&self) -> PathBuf {
        self.base_path.join(&self.data)
    }

    pub fn get_framework(&self) -> PathBuf {
        self.base_path.join(&self.framework)
    }

    fn read_from_directory(dir_path: &Path) -> Result<Self> {
        let file_path = dir_path.join(Self::CONFIG_FILE);
        let read = std::fs::read(file_path)?;
        let mut read: VersionConfig = serde_json::from_slice(&read)?;
        read.base_path = dir_path.to_owned();
        Ok(read)
    }

    fn write_to_directory(&self, dir_path: &Path) -> Result<()> {
        let to_write = serde_json::to_vec_pretty(self)?;
        let file_path = dir_path.join(Self::CONFIG_FILE);
        std::fs::write(file_path, to_write)?;
        Ok(())
    }
}
