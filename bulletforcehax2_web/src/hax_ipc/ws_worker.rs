use futures::channel::mpsc;
use futures::prelude::*;

use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use shared::{C2SMessage, S2CMessage};

#[derive(PartialEq, Eq)]
pub enum OpenState {
    Open,
    Closed,
}

pub struct WsComms {
    send_to_worker: mpsc::UnboundedSender<C2SMessage>,
    recv_from_worker: mpsc::UnboundedReceiver<S2CMessage>,
}

impl WsComms {
    pub fn connect() -> Option<Self> {
        // try to create a ws connection
        let Some(ws) = Self::create_connection() else {
            return None;
        };

        let (send_to_worker, recv_from_comms) = mpsc::unbounded();
        let (send_to_comms, recv_from_worker) = mpsc::unbounded();

        let worker = WsWorker {
            send_to_comms,
            recv_from_comms,
            ws,
        };

        worker.start_task();

        Some(Self {
            send_to_worker,
            recv_from_worker,
        })
    }

    fn create_connection() -> Option<WebSocket> {
        let window = web_sys::window().expect("get window object");
        let host = window.location().host().expect("get origin");
        let url = format!("ws://{host}/hax/ws");

        log::info!("Connecting to websocket at {url}");
        if let Ok(ws) = WebSocket::open(&url) {
            log::debug!("Created ws connection");
            Some(ws)
        } else {
            log::warn!("Failed to create ws connnection");
            None
        }
    }

    pub fn try_recv(&mut self) -> (Vec<S2CMessage>, OpenState) {
        let mut messages = vec![];
        let mut open_state = OpenState::Open;

        while let Ok(msg) = self.recv_from_worker.try_next() {
            match msg {
                Some(msg) => {
                    messages.push(msg);
                }
                None => {
                    log::warn!("Channel to worker closed");
                    open_state = OpenState::Closed;
                    break;
                }
            }
        }

        (messages, open_state)
    }
}

pub struct WsWorker {
    send_to_comms: mpsc::UnboundedSender<S2CMessage>,
    recv_from_comms: mpsc::UnboundedReceiver<C2SMessage>,
    ws: WebSocket,
}

impl WsWorker {
    pub fn start_task(mut self) {
        wasm_bindgen_futures::spawn_local(async move {
            let mut ws = self.ws.fuse();

            loop {
                let msg = futures::select! {
                    from_server = ws.next() => WsWorkerMessage::FromServer(from_server),
                    from_client = self.recv_from_comms.next() => WsWorkerMessage::FromClient(from_client),
                };

                match msg {
                    WsWorkerMessage::FromServer(None) => {
                        log::error!("WebSocket connection unexpectedly closed");
                        break;
                    }
                    WsWorkerMessage::FromServer(Some(Err(e))) => {
                        log::error!("Error while receiving WebSocket message: {e}");
                        break;
                    }
                    WsWorkerMessage::FromServer(Some(Ok(ws_message))) => {
                        let bytes = match ws_message {
                            Message::Bytes(b) => b,
                            Message::Text(t) => t.into_bytes(),
                        };
                        let Ok(parsed) = postcard::from_bytes(&bytes) else {
                            log::error!("Failed to parse incoming ws message, out of sync with backend?");
                            break;
                        };

                        if let Err(e) = self.send_to_comms.unbounded_send(parsed) {
                            log::debug!(
                                "Failed to pass ws message to client, it likely closed: {e}"
                            );
                            break;
                        }
                    }
                    WsWorkerMessage::FromClient(None) => {
                        log::debug!("Channel to worker closed");
                        break;
                    }
                    WsWorkerMessage::FromClient(Some(to_send)) => {
                        let bytes = postcard::to_allocvec(&to_send).expect("c2s to alloc vec");
                        if let Err(e) = ws.send(Message::Bytes(bytes)).await {
                            log::error!("Error while sending ws message: {e}");
                            break;
                        }
                    }
                }
            }

            log::debug!("Closed task for WsWorker");
        });
        log::debug!("Spawned task for WsWorker");
    }
}

enum WsWorkerMessage {
    FromServer(Option<Result<Message, WebSocketError>>),
    FromClient(Option<C2SMessage>),
}
