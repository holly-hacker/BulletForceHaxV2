use std::convert::Infallible;
use std::sync::Arc;

use anyhow::Context;
use http::response::Builder as ResponseBuilder;
use hyper::http;
use hyper::{Body, Request, Response};
use tower::util::BoxCloneService;
use tower::ServiceBuilder;
use tracing::{debug, error};

use crate::config::Config;
use crate::version_manager::VersionConfig;

pub fn create_service(
    version: VersionConfig,
    config: Config,
) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
    let arc = Arc::new((version, config));

    let service = ServiceBuilder::new().service_fn(move |req| handler(req, arc.clone()));

    BoxCloneService::new(service)
}

#[tracing::instrument(name = "AssetServer", level = "info", skip_all, fields(uri = request.uri().path()))]
async fn handler(
    request: Request<Body>,
    arc: Arc<(VersionConfig, Config)>,
) -> Result<Response<Body>, Infallible> {
    match handler_main(request, arc).await {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("Error result while handling asset request {e:?}");
            Ok(Response::builder()
                .status(500)
                .body(format!("Error while handling request: {e:?}").into())
                .expect("should be able to create basic response"))
        }
    }
}

async fn handler_main(
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
        _ => {
            let path = path.strip_prefix("/Build").unwrap_or(path);

            let path = version.get_path(path.trim_start_matches('/'));
            path.exists().then_some(path)
        }
    };

    if let Some(file_path) = file_path {
        debug!("Loading file asset from path {file_path:?}");

        // NOTE: I really wanted to stream this from disk, instead of reading it entirely into memory. I'd have to
        // convert a File into something that implements futures_util::stream::Stream<Item = Result<Bytes, E>> but it
        // seem not too trivial?
        // tokio_util::io::ReaderStream could be an option but it pulls in a bunch of dependencies and I dont know if
        // there will be a performance improvement
        let content = std::fs::read(&file_path)
            .with_context(|| format!("read file {file_path:?} for req {path:?}"))?;

        let builder = ResponseBuilder::new();
        let result = builder
            .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(content.into())?;
        return Ok(result);
    }

    debug!("Cannot find file on asset server, returning 404");

    Ok(ResponseBuilder::new().status(404).body(Body::empty())?)
}
