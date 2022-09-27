use std::sync::mpsc::{channel, Receiver, Sender};

use anyhow::{anyhow, Context, Result};
use tracing::debug;

const UNITY_LOADER_URL: &str = "https://files.crazygames.com/unityloaders/UnityLoader-v3.js";

const POKI_URL: &str = "https://poki.com/en/g/bullet-force-multiplayer";
const POKI_FRAME_URL_PATTERN: &str = r"https://games\.poki\.com/\d+/[\da-f-]+";
const POKI_GAME_URL_PATTERN: &str = r"https://[a-f\d-]+\.poki-gdn\.com/[a-f\d-]+/index\.html";
const POKI_GAME_JSON_PATTERN: &str = r"unityWebglBuildUrl: '([^']+)'";

#[derive(Clone, Copy)]
pub enum FileType {
    UnityLoader,
    GameJson,
    GameFile,
}

pub enum ProgressReport {
    Progress {
        file_type: FileType,
        name: String,
        downloaded: u64,
        total: Option<u64>,
    },
    Finished {
        file_type: FileType,
        name: String,
        data: Vec<u8>,
    },
    Done,
    Crashed(String),
}

pub fn start_download() -> Result<Receiver<ProgressReport>> {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        if let Err(e) = do_download(tx.clone()) {
            _ = tx.send(ProgressReport::Crashed(e.to_string()));
        }
    });

    Ok(rx)
}

fn do_download(tx: Sender<ProgressReport>) -> Result<()> {
    // download loader
    download_file_with_progress(
        UNITY_LOADER_URL,
        "UnityLoader.js",
        FileType::UnityLoader,
        tx.clone(),
    )?;

    // find game files
    let source_1 = ureq::get(POKI_URL).call()?.into_string()?;
    let match_1 = regex::Regex::new(POKI_FRAME_URL_PATTERN)?
        .find(&source_1)
        .ok_or_else(|| anyhow!("Could not find poki regex 1"))?;
    let frame_url = match_1.as_str();
    let source_2 = ureq::get(frame_url)
        .set("referer", "https://poki.com")
        .call()?
        .into_string()?;
    let match_2 = regex::Regex::new(POKI_GAME_URL_PATTERN)?
        .find(&source_2)
        .ok_or_else(|| anyhow!("Could not find poki regex 2"))?;
    let game_url = match_2.as_str();
    let source_3 = ureq::get(game_url)
        .set("referer", "https://poki.com")
        .call()?
        .into_string()?;
    let match_3 = regex::Regex::new(POKI_GAME_JSON_PATTERN)?
        .captures(&source_3)
        .ok_or_else(|| anyhow!("Could not find poki regex 3"))?
        .get(1)
        .ok_or_else(|| anyhow!("Could not find group in poki regex 3"))?;
    let rel_url_json = match_3.as_str().replace("\\/", "/");

    let abs_url_json_base = dbg!(&game_url[..game_url.rfind('/').unwrap()]);
    let abs_url_json = abs_url_json_base.to_string() + "/" + &rel_url_json;
    download_file_with_progress(&abs_url_json, &rel_url_json, FileType::GameJson, tx.clone())?;

    // yes I'm downloading the json twice, I cba to rewrite the code
    let json = ureq::get(&abs_url_json).call()?.into_string()?;
    let json: serde_json::Value = serde_json::from_str(&json).context("parse game json")?;

    let base_url_file = dbg!(&abs_url_json[..abs_url_json.rfind('/').unwrap()]);
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
        download_file_with_progress(&abs_url_file, file_name, FileType::GameFile, tx.clone())?;
    }

    tx.send(ProgressReport::Done)?;

    Ok(())
}

fn download_file_with_progress(
    url: &str,
    name: &str,
    file_type: FileType,
    tx: Sender<ProgressReport>,
) -> Result<()> {
    let response = ureq::get(url).call()?;
    let content_length = response
        .header("content-length")
        .and_then(|len| len.parse::<u64>().ok());

    use std::io::Read;

    let mut reader = response.into_reader();
    let mut data = vec![];

    tx.send(ProgressReport::Progress {
        file_type,
        name: name.to_string(),
        downloaded: 0,
        total: content_length,
    })?;
    let mut buffer = [0u8; 1024 * 256];
    loop {
        let len = reader.read(&mut buffer)?;
        data.extend_from_slice(&buffer[..len]);

        if len != 0 {
            tx.send(ProgressReport::Progress {
                file_type,
                name: name.to_string(),
                downloaded: data.len() as u64,
                total: content_length,
            })?;
        } else {
            tx.send(ProgressReport::Finished {
                file_type,
                name: name.to_string(),
                data,
            })?;
            break;
        }
    }

    debug!("Downloaded {name} from {url}");
    Ok(())
}
