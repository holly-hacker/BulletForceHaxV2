use std::path::{Path, PathBuf};

use anyhow::Context;
use bulletforcehax2_lib::version_scraper::{self, FileType};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

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

    pub async fn get_or_download(base_path: &Path) -> anyhow::Result<VersionConfig> {
        std::fs::create_dir_all(base_path)
            .with_context(|| format!("create directories for config: {base_path:?}"))?;

        // if a version already exists, return that
        if let Some(version) = VersionConfig::read_from_directory(base_path)
            .context("try read existing version config")?
        {
            debug!("existing game version found");
            return Ok(version);
        }

        // download a new
        warn!("No local game version found, downloading new version");
        let downloaded = do_download(base_path)
            .await
            .context("download new game version")?;

        Ok(downloaded)
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

    fn read_from_directory(dir_path: &Path) -> anyhow::Result<Option<Self>> {
        let file_path = dir_path.join(Self::CONFIG_FILE);

        if !file_path.exists() {
            return Ok(None);
        }

        let read = std::fs::read(file_path)?;
        let mut read: VersionConfig = serde_json::from_slice(&read)?;
        read.base_path = dir_path.to_owned();
        Ok(Some(read))
    }

    fn write_to_directory(&self, dir_path: &Path) -> anyhow::Result<()> {
        let to_write = serde_json::to_vec_pretty(self)?;
        let file_path = dir_path.join(Self::CONFIG_FILE);
        std::fs::write(file_path, to_write)?;
        Ok(())
    }
}

async fn do_download(base_path: &Path) -> anyhow::Result<VersionConfig> {
    // TODO: this scraper is quite annoying to use, as it's built for use in UI
    // it should be rewritten
    let mut rx_scraper = version_scraper::start_download_thread()?;

    let mut loader = None;
    let mut code = None;
    let mut data = None;
    let mut framework = None;

    while let Some(report) = rx_scraper.recv().await {
        match report {
            version_scraper::ProgressReport::Progress {
                file_type,
                name,
                downloaded,
                total: _,
            } => {
                debug!(
                    "Download progress on {name} ({file_type:?}): {}",
                    bytesize::ByteSize(downloaded)
                )
            }
            version_scraper::ProgressReport::FileDownloaded {
                file_type,
                name,
                data: file_data,
            } => {
                info!("Finished downloading {name}");
                let path = base_path.join(&name);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, file_data)?;

                match file_type {
                    FileType::UnityLoader => loader = Some(name),
                    FileType::Framework => framework = Some(name),
                    FileType::Code => code = Some(name),
                    FileType::Data => data = Some(name),
                }
            }
            version_scraper::ProgressReport::AllFilesDownloaded => {
                debug!("all files downloaded");
                break;
            }
            version_scraper::ProgressReport::Crashed(msg) => {
                error!("an error occured while downloading: {msg}");
                anyhow::bail!(msg);
            }
        }
    }

    let config = VersionConfig {
        base_path: base_path.to_owned(),
        unity_loader: loader.context("did not find unity loader")?,
        code: code.context("did not find code file")?,
        data: data.context("did not find data file")?,
        framework: framework.context("did not find framework file")?,
    };

    config.write_to_directory(base_path)?;

    Ok(config)
}
