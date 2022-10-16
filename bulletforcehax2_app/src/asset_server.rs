use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};

use anyhow::Context;
use http::response::Builder as ResponseBuilder;
use hyper::http;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tracing::{debug, error};

use crate::config::Config;
use crate::version_manager::VersionConfig;

pub async fn start_asset_server(version: VersionConfig, config: Config, port: u16) {
    tokio::spawn(async move { block_on_server(version, config, port).await });
}

async fn block_on_server(version: VersionConfig, config: Config, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let arc = Arc::new((version, config));

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let arc = arc.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handler(req, arc.clone()))) }
    }));

    debug!("asset http server created");
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}

#[tracing::instrument(name = "AssetServer", level = "info", skip_all, fields(uri = request.uri().path()))]
async fn handler(
    request: Request<Body>,
    arc: Arc<(VersionConfig, Config)>,
) -> anyhow::Result<Response<Body>> {
    let path = request.uri().path();
    debug!("Incoming request for asset server");

    let (version, config) = arc.as_ref();

    if path == "/" {
        const INDEX: &[u8] = include_bytes!("../assets/index.html");
        return Ok(ResponseBuilder::new().body(INDEX.into())?);
    }

    if path == "/config.json" {
        let data = serde_json::to_vec(config).unwrap();
        return Ok(ResponseBuilder::new()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(data.into())?);
    }

    let file_path = match path {
        "/Build/$$game$$.json" => Some(version.get_game_json()),
        "/$$loader$$.js" => Some(version.get_unity_loader()),
        _ if path.starts_with("/Build/") => Some(version.get_path(path.trim_start_matches('/'))),
        _ => None,
    };

    if let Some(file_path) = file_path {
        debug!("Loading file asset from path {file_path:?}");

        // NOTE: I really wanted to stream this from disk, instead of reading it entirely into memory. I'd have to
        // convert a File into something that implements futures_util::stream::Stream<Item = Result<Bytes, E>> but it
        // seem not too trivial?
        let content = std::fs::read(&file_path)
            .with_context(|| format!("read file {:?} for req {:?}", file_path, path))?;

        let builder = ResponseBuilder::new();
        let result = builder
            .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(content.into())?;
        return Ok(result);
    }

    debug!("Cannot find file on asset server, returning 404");

    Ok(ResponseBuilder::new().status(404).body(Body::empty())?)
}
