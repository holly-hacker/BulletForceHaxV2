pub mod version_scraper;
mod webrequest_proxy;
mod websocket_proxy;

/// temp function to enable webrequest proxy
pub fn init_webrequest_proxy() {
    tokio::spawn(async move {
        webrequest_proxy::block_on_server().await;
    });
}

/// temp function to enable websocket proxy
pub fn init_websocket_proxy() {
    tokio::spawn(async move {
        websocket_proxy::block_on_server().await;
    });
}
