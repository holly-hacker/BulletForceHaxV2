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
use shared::{C2SMessage, S2CMessage};
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

async fn read(mut receiver: SplitStream<WebSocket>, hax_state: HaxSharedState) {
    while let Some(next) = receiver.next().await {
        let next = next.expect("get next message");
        match next {
            Message::Text(msg) => panic!("Received unexpected text message: {msg}"),
            Message::Binary(msg) => {
                let data: C2SMessage = postcard::from_bytes(&msg).unwrap();
                match data {
                    C2SMessage::UpdateSettings(settings) => {
                        info!("Received updated settings from client");
                        let mut hax_state = hax_state.lock().await;
                        hax_state.settings = settings;
                    }
                    C2SMessage::Command => todo!("command"),
                }
            }
            Message::Close(_) => todo!("on connection close"),
            _ => (),
        }
    }
}

// TODO: make this reactive rather than time-based
async fn write(mut sender: SplitSink<WebSocket, Message>, hax_state: HaxSharedState) {
    let (state, settings) = {
        let hax_state = hax_state.lock().await;
        (
            hax_state.copy_to_network_update(),
            hax_state.settings.clone(),
        )
    };
    let message = S2CMessage::InitialState(state, settings);
    let state_bytes = postcard::to_allocvec(&message).expect("serialize network message");

    sender
        .send(Message::Binary(state_bytes))
        .await
        .expect("send item over websocket");

    loop {
        let state = hax_state.lock().await.copy_to_network_update();
        let message = S2CMessage::NewGameState(state);
        let state_bytes = postcard::to_allocvec(&message).expect("serialize network message");

        sender
            .send(Message::Binary(state_bytes))
            .await
            .expect("send item over websocket");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
