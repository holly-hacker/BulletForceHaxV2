//! Provides a method to download the latest version of the game files.

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use hyper::{body::HttpBody, Body, Client, Request};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::decompression::{Decompression, DecompressionLayer};
use tracing::debug;

const UNITY_LOADER_URL: &str = "https://files.crazygames.com/unityloaders/UnityLoader-v3.js";
const CG_FRAME_URL: &str = "https://games.crazygames.com/en_US/bullet-force-multiplayer/index.html";
const CG_JSON_PATTERN: &str = r#"moduleJsonUrl":"([^"]+)"#;

/// Describes the role of a downloading file.
#[derive(Clone, Copy)]
pub enum FileType {
    UnityLoader,
    GameJson,
    GameFile,
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

    // download loader
    download_file_with_progress(
        &mut client,
        UNITY_LOADER_URL,
        "UnityLoader.js",
        FileType::UnityLoader,
        tx.clone(),
    )
    .await?;

    // find game files
    let source_1 = hyper_get(&mut client, CG_FRAME_URL)
        .await
        .context("get source_1")?;
    let match_1 = regex::Regex::new(CG_JSON_PATTERN)?
        .captures(&source_1)
        .ok_or_else(|| anyhow!("Could not find CG regex 1"))?
        .get(1)
        .ok_or_else(|| anyhow!("Could not find group in CG regex 1"))?;
    let abs_url_json = match_1.as_str();
    let rel_url_json = abs_url_json.split('/').last().ok_or_else(|| {
        anyhow!("Could not split json file url. Found something invalid? '{abs_url_json}'")
    })?;
    let abs_url_json_base = &abs_url_json[..(abs_url_json.len() - rel_url_json.len() - 1)]; // don't include /

    // TODO: this can happen in parallel
    download_file_with_progress(
        &mut client,
        abs_url_json,
        rel_url_json,
        FileType::GameJson,
        tx.clone(),
    )
    .await?;

    // yes I'm downloading the json twice, I cba to rewrite the code
    let json = hyper_get(&mut client, abs_url_json).await?;
    let json: serde_json::Value = serde_json::from_str(&json).context("parse game json")?;

    let base_url_file = &abs_url_json[..abs_url_json.rfind('/').unwrap()];
    // TODO: can happen in parallel
    for rel_url in [
        json["dataUrl"]
            .as_str()
            .ok_or_else(|| anyhow!("get `dataUrl` field in json"))?,
        json["wasmCodeUrl"]
            .as_str()
            .ok_or_else(|| anyhow!("get `wasmCodeUrl` field in json"))?,
        json["wasmFrameworkUrl"]
            .as_str()
            .ok_or_else(|| anyhow!("get `wasmFrameworkUrl` field in json"))?,
    ] {
        let abs_url_file = format!("{base_url_file}/{rel_url}");
        let file_name = &abs_url_file[(abs_url_json_base.len() + 1)..];
        download_file_with_progress(
            &mut client,
            &abs_url_file,
            file_name,
            FileType::GameFile,
            tx.clone(),
        )
        .await?;
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
