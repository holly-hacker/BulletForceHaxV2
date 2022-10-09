//! Provides a method to download the latest version of the game files.

use std::time::{Duration, Instant};

use hyper::{body::HttpBody, Body, Client, Request};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use anyhow::{anyhow, Context, Result};
use tracing::debug;

const UNITY_LOADER_URL: &str = "https://files.crazygames.com/unityloaders/UnityLoader-v3.js";

const POKI_URL: &str = "https://poki.com/en/g/bullet-force-multiplayer";
const POKI_FRAME_URL_PATTERN: &str = r"https://games\.poki\.com/\d+/[\da-f-]+";
const POKI_GAME_URL_PATTERN: &str = r"https://[a-f\d-]+\.poki-gdn\.com/[a-f\d-]+/index\.html";
const POKI_GAME_JSON_PATTERN: &str = r"unityWebglBuildUrl: '([^']+)'";

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
    // TODO: add tower-http's compressing middleware
    let client = Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());

    // download loader
    download_file_with_progress(
        &client,
        UNITY_LOADER_URL,
        "UnityLoader.js",
        FileType::UnityLoader,
        tx.clone(),
    )
    .await?;

    // find game files
    let source_1 = hyper_get(&client, POKI_URL).await.context("get source_1")?;
    let match_1 = regex::Regex::new(POKI_FRAME_URL_PATTERN)?
        .find(&source_1)
        .ok_or_else(|| anyhow!("Could not find poki regex 1"))?;
    let frame_url = match_1.as_str();
    let source_2 = hyper_get_with_referrer(&client, frame_url)
        .await
        .context("get source_2")?;
    let match_2 = regex::Regex::new(POKI_GAME_URL_PATTERN)?
        .find(&source_2)
        .ok_or_else(|| anyhow!("Could not find poki regex 2"))?;
    let game_url = match_2.as_str();
    let source_3 = hyper_get_with_referrer(&client, game_url)
        .await
        .context("get source_3")?;
    let match_3 = regex::Regex::new(POKI_GAME_JSON_PATTERN)?
        .captures(&source_3)
        .ok_or_else(|| anyhow!("Could not find poki regex 3"))?
        .get(1)
        .ok_or_else(|| anyhow!("Could not find group in poki regex 3"))?;
    let rel_url_json = match_3.as_str().replace("\\/", "/");

    let abs_url_json_base = &game_url[..game_url.rfind('/').unwrap()];
    let abs_url_json = abs_url_json_base.to_string() + "/" + &rel_url_json;
    download_file_with_progress(
        &client,
        &abs_url_json,
        &rel_url_json,
        FileType::GameJson,
        tx.clone(),
    )
    .await?;

    // yes I'm downloading the json twice, I cba to rewrite the code
    let json = hyper_get(&client, &abs_url_json).await?;
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
            &client,
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

async fn hyper_get<T>(client: &Client<T>, url: &str) -> Result<String>
where
    T: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let uri = hyper::Uri::try_from(url)?;
    let response = client.get(uri).await?;
    let bytes = hyper::body::to_bytes(response).await?;
    Ok(String::from_utf8(bytes.to_vec())?)
}

async fn hyper_get_with_referrer<T>(client: &Client<T>, url: &str) -> Result<String>
where
    T: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let uri = hyper::Uri::try_from(url)?;
    let request = Request::builder()
        .header("referer", "https://poki.com")
        .uri(uri)
        .body(Body::empty())?;
    let response = client.request(request).await?;
    let bytes = hyper::body::to_bytes(response).await?;
    Ok(String::from_utf8(bytes.to_vec())?)
}

async fn download_file_with_progress<T>(
    client: &Client<T>,
    url: &str,
    name: &str,
    file_type: FileType,
    tx: Sender<ProgressReport>,
) -> Result<()>
where
    T: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let mut response = client.get(hyper::Uri::try_from(url).unwrap()).await?;
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|len| len.to_str().unwrap().parse::<u64>().ok());

    let mut data = vec![];

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

    const UPDATE_FREQUENCY: Duration = Duration::from_millis(50);
    let mut last_progress = Instant::now();
    while let Some(next) = response.data().await {
        let next = next?;
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
