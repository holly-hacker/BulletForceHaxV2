use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
};

static DIST_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../dist");

pub async fn serve(req: Request<Body>) -> Response<Body> {
    let path = req.uri().path().trim_start_matches('/');

    let file = DIST_DIR.get_file(path);

    let Some(file) = file else {
        // file not found, serving index file and let client-side router take care of it
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
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
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(file.contents()))
        .expect("response")
}
