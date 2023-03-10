mod game_assets;

use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    routing::get,
    Router,
};

pub fn get_router() -> Router {
    Router::new()
        .route("/game_assets/:file", get(game_assets::handle))
        .fallback_service(get(serve_frontend))
}

async fn serve_frontend(req: Request<Body>) -> Response<Body> {
    let path = req.uri().path().trim_start_matches('/');

    let file = crate::DIST_DIR.get_file(path);

    let Some(file) = file else {
        // file not found, serving index file and let client-side router take care of it
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(crate::DIST_DIR.get_file("index.html").expect("read index file").contents()))
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
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(file.contents()))
        .expect("response")
}
