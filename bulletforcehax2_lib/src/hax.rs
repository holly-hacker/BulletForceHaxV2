//! The main module of BulletForceHaxV2.

use std::sync::Arc;

use futures_util::lock::Mutex;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info, warn};

use crate::proxy::websocket_proxy::WebSocketProxy;

/// An instance of BulletForceHaxV2. It handles the webrequest and websocket proxies as well as the internal state.
///
/// # Remarks
/// It is not recommended to create multiple instances of this because [BulletForceHax::start_websocket_proxy] and
/// [BulletForceHax::start_webrequest_proxy] can only be run once, as they bind to a pre-defined port.
#[derive(Default)]
pub struct BulletForceHax {
    webrequest_proxy: Option<()>,
    websocket_proxy: Option<()>,
    state: Arc<futures_util::lock::Mutex<HaxState>>,
}

/// The internal state.
#[derive(Default)]
pub struct HaxState {
    lobby_socket: Option<WebSocketProxy>,
    gameplay_socket: Option<WebSocketProxy>,
}

impl BulletForceHax {
    /// Creates the webrequest proxy handler thread. Panics if one has already been created.
    pub fn start_webrequest_proxy(&mut self) {
        if self.webrequest_proxy.is_some() {
            panic!("webrequest proxy is already enabled");
        }

        tokio::spawn(async move {
            crate::proxy::webrequest_proxy::block_on_server().await;
        });

        self.webrequest_proxy = Some(());
    }

    /// Creates the websocket proxy handler thread. Panics if one has already been created.
    pub fn start_websocket_proxy(&mut self) {
        if self.websocket_proxy.is_some() {
            panic!("websocket proxy is already enabled");
        }

        let (new_connection_send, new_connection_recv) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            // start the proxy
            crate::proxy::websocket_proxy::block_on_server(Some(new_connection_send)).await
        });

        let state = self.state.clone();
        tokio::spawn(async move {
            Self::store_new_connections_in_state_vars(state, new_connection_recv).await;
        });

        self.websocket_proxy = Some(());
    }

    // bookkeeping to ensure the websocket connection gets written and unwritten to the right variable
    async fn store_new_connections_in_state_vars(
        state: Arc<Mutex<HaxState>>,
        mut new_connection_recv: Receiver<WebSocketProxy>,
    ) {
        while let Some(mut conn) = new_connection_recv.recv().await {
            match conn.get_port() {
                // lobby
                2053 => {
                    let notify_closed = conn.take_notify_closed();

                    let state = state.clone();
                    {
                        let mut locked_state = state.lock().await;
                        if let Some(_) = locked_state.lobby_socket {
                            warn!("lobby socket connection created while one already existed! did it not get cleared correctly?");
                        }
                        locked_state.lobby_socket = Some(conn);
                    }

                    match notify_closed {
                        Some(n) => {
                            // create task to clear the socket variable when the connection dies
                            tokio::spawn(async move {
                                // wait for the socket to close
                                n.notified().await;

                                info!("lobby websocket closed");
                                let mut locked_state = state.lock().await;
                                if locked_state.lobby_socket.is_none() {
                                    warn!("lobby socket connection was closed but it did not exist yet");
                                }
                                locked_state.lobby_socket = None;
                            });
                        }
                        None => warn!("A lobby websocket task was created but no closed Notify was found. Detecting socket closing will not work"),
                    }
                }
                // gameplay
                2083 => {
                    let notify_closed = conn.take_notify_closed();

                    let state = state.clone();
                    {
                        let mut locked_state = state.lock().await;
                        if let Some(_) = locked_state.gameplay_socket {
                            warn!("gameplay socket connection created while one already existed! did it not get cleared correctly?");
                        }
                        locked_state.gameplay_socket = Some(conn);
                    }

                    match notify_closed {
                        Some(n) => {
                            // create task to clear the socket variable when the connection dies
                            tokio::spawn(async move {
                                // wait for the socket to close
                                n.notified().await;

                                info!("gameplay websocket closed");
                                let mut locked_state = state.lock().await;
                                if locked_state.gameplay_socket.is_none() {
                                    warn!("gameplay socket connection was closed but it did not exist yet");
                                }
                                locked_state.gameplay_socket = None;
                            });
                        }
                        None => warn!("A gameplay websocket task was created but no closed Notify was found. Detecting socket closing will not work"),
                    }
                }
                p => warn!("WebSocket connection initiated over unknown target port {p}"),
            }
        }

        debug!("websocket proxy receiver closed");
    }
}
