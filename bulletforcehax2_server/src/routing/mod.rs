mod game_assets;
mod hax_ipc_server;

#[cfg(not(feature = "read-from-disk"))]
mod web_assets;

#[cfg(feature = "read-from-disk")]
mod web_assets_live;

use std::sync::Arc;

use axum::{routing::get, Extension, Json, Router};
#[cfg(not(feature = "read-from-disk"))]
use web_assets::serve;
#[cfg(feature = "read-from-disk")]
use web_assets_live::serve;

use crate::config::Config;

pub fn get_router() -> Router {
    Router::new()
        .route("/config.json", get(get_config))
        .route("/hax/ws", get(hax_ipc_server::handle))
        .route("/game_assets/:file", get(game_assets::handle))
        .fallback_service(get(serve))
}

async fn get_config(Extension(config): Extension<Arc<Config>>) -> Json<Config> {
    Json((*config).clone())
}
