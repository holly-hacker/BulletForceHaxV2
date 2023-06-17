mod game_assets;
mod hax_ipc_server;

#[cfg(not(feature = "read-from-disk"))]
mod web_assets;

#[cfg(feature = "read-from-disk")]
mod web_assets_live;

use std::sync::Arc;

use axum::{
    http::{header, HeaderMap},
    response::Html,
    routing::get,
    Extension, Json, Router,
};
#[cfg(not(feature = "read-from-disk"))]
use web_assets::serve as web_assets_serve;
#[cfg(feature = "read-from-disk")]
use web_assets_live::serve as web_assets_serve;

use crate::config::Config;

pub fn get_router() -> Router {
    Router::new()
        .route("/config.json", get(get_config))
        .route("/hax/ws", get(hax_ipc_server::handle))
        .route("/game_assets/:file", get(game_assets::handle))
        .route("/game", get(get_game_html))
        .route("/script.js", get(get_game_script))
        .fallback_service(get(web_assets_serve))
}

async fn get_config(Extension(config): Extension<Arc<Config>>) -> Json<Config> {
    Json((*config).clone())
}

async fn get_game_html() -> Html<&'static str> {
    Html(include_str!("../../assets/index.html"))
}

async fn get_game_script() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        "application/javascript".parse().unwrap(),
    );
    (headers, include_str!("../../assets/script.js"))
}
