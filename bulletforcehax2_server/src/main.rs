mod config;
mod routing;
mod utils;
mod version_management;

use std::{net::SocketAddr, sync::Arc};

use axum::{routing::any_service, Extension};
use bulletforcehax2_lib::hax::BulletForceHax;
use config::Config;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};

use crate::{routing::get_router, version_management::VersionConfig};

#[tokio::main]
async fn main() {
    // read config from cli args
    let config = config::get_config();

    // initialize logging
    let _guard = utils::init_logging(&config);
    debug!(config = format!("{config:?}"), "Loaded config");

    // get cached game version, or download it if uncached
    let version_config = match VersionConfig::get_or_download(&config.game_dir).await {
        Ok(x) => x,
        Err(e) => {
            error!("Error while trying to get game info: {e:?}");
            return;
        }
    };

    // run the http server
    // this serves the web ui and will host the proxy endpoints
    run_server(config, version_config).await
}

async fn run_server(config: Config, version_config: VersionConfig) {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    // initialize hax
    let mut hax = BulletForceHax::default();
    let state = hax.get_state();

    let router = get_router()
        .layer(CorsLayer::permissive())
        .route("/request", any_service(hax.get_webrequest_proxy()))
        .route("/socket", any_service(hax.get_websocket_proxy()))
        .layer(Extension(Arc::new(config)))
        .layer(Extension(Arc::new(version_config)))
        .layer(Extension(state));

    info!("binding on http://localhost:{}", addr.port());
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
