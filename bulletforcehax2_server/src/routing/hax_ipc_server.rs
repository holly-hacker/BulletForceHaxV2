use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use bulletforcehax2_lib::hax::HaxSharedState;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tracing::info;

pub async fn handle(
    ws: WebSocketUpgrade,
    Extension(hax_state): Extension<HaxSharedState>,
) -> Response {
    info!("Received WebSocket connection");
    ws.on_upgrade(|x| handle_websocket(x, hax_state))
}

async fn handle_websocket(socket: WebSocket, hax_state: HaxSharedState) {
    let (sender, receiver) = socket.split();

    tokio::spawn(write(sender, hax_state.clone()));
    tokio::spawn(read(receiver, hax_state));
}

async fn read(mut receiver: SplitStream<WebSocket>, _hax_state: HaxSharedState) {
    while let Some(next) = receiver.next().await {
        let next = next.expect("get next message");
        match next {
            Message::Text(msg) => info!("Received text message: {msg}"),
            Message::Binary(msg) => info!("Received binary message: {msg:?}"),
            Message::Close(_) => todo!(),
            _ => (),
        }
    }
}

async fn write(mut sender: SplitSink<WebSocket, Message>, _hax_state: HaxSharedState) {
    for i in 0.. {
        info!("Sending item {i}");
        sender
            .send(Message::Text(format!("hello, world! {i}")))
            .await
            .expect("send item over websocket");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
