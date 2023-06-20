use std::path::PathBuf;

use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
};

pub async fn serve(req: Request<Body>) -> Response<Body> {
    // TODO: check for local file inclusion vulnerability
    let path = req.uri().path().trim_start_matches('/');

    let root = std::env!("CARGO_MANIFEST_DIR");
    let directory = PathBuf::from(format!("{root}/../dist"))
        .canonicalize()
        .expect("canonicalize path");

    let mut file_path = directory.clone();
    file_path.push(path);
    // let file = crate::DIST_DIR.get_file(path);
    let file = std::fs::read(file_path).ok();

    let Some(file) = file else {
        // file not found, serving index file and let client-side router take care of it
        let mut file_path = directory;
        file_path.push("index.html");
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(std::fs::read(file_path).expect("read index file")))
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
        .body(Body::from(file))
        .expect("response")
}
