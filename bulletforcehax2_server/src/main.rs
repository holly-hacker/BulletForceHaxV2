mod config;
mod utils;
mod version_management;

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    routing::get,
    Router,
};
use config::Config;
use include_dir::Dir;
use std::net::SocketAddr;
use tracing::{debug, error, info};

use crate::version_management::VersionConfig;

static DIST_DIR: Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../dist");

#[tokio::main]
async fn main() {
    // read config from cli args
    let config = config::get_config();

    // initialize logging
    let _guard = utils::init_logging(&config);
    debug!(config = format!("{config:?}"), "Loaded config");

    // get cached game version, or download it if uncached
    let _version_info = match VersionConfig::get_or_download(&config.game_dir).await {
        Ok(x) => x,
        Err(e) => {
            error!("Error while trying to get game info: {e:?}");
            return;
        }
    };

    // run the http server
    // this serves the web ui and will host the proxy endpoints
    run_server(&config).await
}

async fn run_server(config: &Config) {
    let router = Router::new()
        .route("/test-api", get(test))
        .fallback_service(get(|req: Request<Body>| async move {
            load_from_disk(req).await
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    info!("binding on http://{addr}");
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn test() -> String {
    DIST_DIR
        .files()
        .map(|x| format!("{} ", x.path().to_string_lossy()))
        .collect()
}

async fn load_from_disk(req: Request<Body>) -> Response<Body> {
    let path = req.uri().path().trim_start_matches('/');

    let file = DIST_DIR.get_file(path);

    let Some(file) = file else {
        // file not found, serving index file and let client-side router take care of it
        return Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(DIST_DIR.get_file("index.html").expect("read index file").contents()))
        .expect("response");
    };

    let content_type = match path.split('.').last() {
        Some("wasm") => "application/wasm",
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        _ => "text/plain",
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .body(Body::from(file.contents()))
        .expect("response")
}
