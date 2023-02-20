//! Provides a method to download the latest version of the game files.

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use hyper::{body::HttpBody, Body, Client, Request};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::decompression::{Decompression, DecompressionLayer};
use tracing::debug;

const CG_FRAME_URL: &str = "https://games.crazygames.com/en_US/bullet-force-multiplayer/index.html";
const CG_JSON_PATTERN: &str = "(?m)^var options = (.+);$";

/// Describes the role of a downloading file.
#[derive(Clone, Copy)]
pub enum FileType {
    UnityLoader,
    Framework,
    Code,
    Data,
}

/// Indicates progress on a downloading file
pub enum ProgressReport {
    /// Indicates that a file is being downloaded and lists the download progress.
    Progress {
        file_type: FileType,
        name: String,
        downloaded: u64,
        total: Option<u64>,
    },
    /// Indicates that a file has finished downloading.
    FileDownloaded {
        file_type: FileType,
        name: String,
        data: Vec<u8>,
    },
    /// Indicates that all files have been downloaded and the download process has finished.
    AllFilesDownloaded,
    /// Indicates that something went wrong with the download and the download process has been aborted.
    Crashed(String),
}

/// Starts a thread that scrapes and downloads the game files. It returns a receiver that produces progress info.
pub fn start_download_thread() -> Result<Receiver<ProgressReport>> {
    let (tx, rx) = channel(128);

    tokio::spawn(async move {
        let tx = tx.clone();
        if let Err(e) = do_download(tx.clone()).await {
            tracing::error!("Error while downloading version: {e}");
            _ = tx.send(ProgressReport::Crashed(e.to_string())).await;
        }
    });

    Ok(rx)
}

async fn do_download(tx: Sender<ProgressReport>) -> Result<()> {
    let client = Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
    let mut client = ServiceBuilder::new()
        .layer(DecompressionLayer::new())
        .service(client);

    // find game files
    let frame_source = hyper_get(&mut client, CG_FRAME_URL)
        .await
        .context("get source_1")?;
    let json_match = regex::Regex::new(CG_JSON_PATTERN)?
        .captures(&frame_source)
        .ok_or_else(|| anyhow!("Could not find json regex"))?
        .get(1)
        .ok_or_else(|| anyhow!("Could not find group in json regex"))?;
    let json_string = json_match.as_str();
    let json_obj: serde_json::Value = serde_json::from_str(json_string)?;

    let loader_options = json_obj["loaderOptions"]
        .as_object()
        .context("get .loaderOptions")?;
    let config_options = loader_options["unityConfigOptions"]
        .as_object()
        .context("get .loaderOptions.unityConfigOptions")?;

    let url_loader = loader_options["unityLoaderUrl"]
        .as_str()
        .context("get .loaderOptions.unityLoaderUrl")?;
    let url_code = config_options["codeUrl"]
        .as_str()
        .context("get .loaderOptions.unityConfigOptions.codeUrl")?;
    let url_data = config_options["dataUrl"]
        .as_str()
        .context("get .loaderOptions.unityConfigOptions.dataUrl")?;
    let url_framework = config_options["frameworkUrl"]
        .as_str()
        .context("get .loaderOptions.unityConfigOptions.frameworkUrl")?;

    // TODO: can happen in parallel
    for (url, file_type) in [
        (url_loader, FileType::UnityLoader),
        (url_code, FileType::Code),
        (url_data, FileType::Data),
        (url_framework, FileType::Framework),
    ] {
        let file_name = url.split('/').last().unwrap();
        download_file_with_progress(&mut client, url, file_name, file_type, tx.clone()).await?;
    }

    tx.send(ProgressReport::AllFilesDownloaded)
        .await
        .ok()
        .context("all files downloaded send")
        .unwrap();

    Ok(())
}

async fn hyper_get<T>(client: &mut Decompression<Client<T>>, url: &str) -> Result<String>
where
    T: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let uri = hyper::Uri::try_from(url)?;
    let request = Request::builder().uri(uri).body(Body::empty())?;
    let response = client.call(request).await?;
    let bytes = hyper::body::to_bytes(response)
        .await
        .map_err(|e| anyhow!("Error while getting http: {}", e))?;
    Ok(String::from_utf8(bytes.to_vec())?)
}

async fn download_file_with_progress<T>(
    client: &mut Decompression<Client<T>>,
    url: &str,
    name: &str,
    file_type: FileType,
    tx: Sender<ProgressReport>,
) -> Result<()>
where
    T: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let uri = hyper::Uri::try_from(url)?;
    let request = Request::builder().uri(uri).body(Body::empty())?;
    let mut response = client.ready().await?.call(request).await?;

    let size_hint = response.body().size_hint();
    let content_length = size_hint.exact();

    tx.send(ProgressReport::Progress {
        file_type,
        name: name.to_string(),
        downloaded: 0,
        total: content_length,
    })
    .await
    .ok()
    .context("initial progress send")
    .unwrap();

    let mut data = Vec::with_capacity(size_hint.lower() as usize);

    const UPDATE_FREQUENCY: Duration = Duration::from_millis(50);
    let mut last_progress = Instant::now();
    while let Some(next) = response.data().await {
        let next = next.map_err(|e| anyhow!("Error while reading http body: {}", e))?;
        data.extend_from_slice(&next);

        let now = Instant::now();
        if (now - last_progress) > UPDATE_FREQUENCY {
            tx.send(ProgressReport::Progress {
                file_type,
                name: name.to_string(),
                downloaded: data.len() as u64,
                total: content_length,
            })
            .await
            .ok()
            .context("progress send")
            .unwrap();

            last_progress = now;
        }
    }

    // kind of a hack
    tx.send(ProgressReport::Progress {
        file_type,
        name: name.to_string(),
        downloaded: data.len() as u64,
        total: content_length,
    })
    .await
    .ok()
    .context("final progress send")
    .unwrap();

    tx.send(ProgressReport::FileDownloaded {
        file_type,
        name: name.to_string(),
        data,
    })
    .await
    .ok()
    .context("file downloaded send")
    .unwrap();

    debug!("Downloaded {name} from {url}");
    Ok(())
}
