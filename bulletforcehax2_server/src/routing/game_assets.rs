use std::sync::Arc;

use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    Extension,
};
use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

use crate::version_management::VersionConfig;

pub async fn handle(
    Path(file): Path<String>,
    Extension(version_config): Extension<Arc<VersionConfig>>,
) -> (
    StatusCode,
    HeaderMap,
    StreamBody<ReaderStream<Box<dyn AsyncRead + Send + Unpin>>>,
) {
    let (content_type, path) = match file.as_str() {
        "loader.js" => (
            "application/javascript",
            version_config
                .get_unity_loader()
                .to_string_lossy()
                .into_owned(),
        ),
        "framework.js" => (
            "application/javascript",
            version_config
                .get_framework()
                .to_string_lossy()
                .into_owned(),
        ),
        "code.wasm" => (
            "application/wasm",
            version_config.get_code().to_string_lossy().into_owned(),
        ),
        "data.data" => (
            "application/octet-stream",
            version_config.get_data().to_string_lossy().into_owned(),
        ),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                HeaderMap::default(),
                StreamBody::new(ReaderStream::new(Box::new(b"not found".as_slice()))),
            );
        }
    };

    let mut headers = HeaderMap::new();
    headers.append(header::CONTENT_TYPE, content_type.parse().unwrap());

    let file = tokio::fs::File::open(path)
        .await
        .expect("read game asset from disk");
    let file: Box<dyn AsyncRead + Send + Unpin> = Box::new(file);
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    (StatusCode::OK, headers, body)
}
