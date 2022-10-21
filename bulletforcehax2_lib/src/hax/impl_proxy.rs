use std::{convert::Infallible, sync::Arc};

use futures_util::lock::Mutex;
use hyper::{Body, Request, Response};
use tokio::sync::mpsc::Receiver;
use tower::util::BoxCloneService;
use tracing::{debug, info, warn};

use super::{BulletForceHax, HaxState};
use crate::{
    hax::{GameplayState, LobbyState},
    proxy::{websocket_proxy::WebSocketProxy, WebSocketServer},
};

impl BulletForceHax {
    pub fn get_webrequest_proxy(
        &mut self,
    ) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        crate::proxy::webrequest_proxy::create_service(self.state.clone())
    }

    pub fn get_websocket_proxy(
        &mut self,
    ) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        let (new_connection_send, new_connection_recv) = tokio::sync::mpsc::channel(4);

        let state = self.state.clone();
        tokio::spawn(async move {
            Self::store_new_connections_in_state_vars(state, new_connection_recv).await;
        });

        crate::proxy::websocket_proxy::create_service(new_connection_send, self.state.clone())
    }

    // bookkeeping to ensure the websocket connection gets written and unwritten to the right variable
    async fn store_new_connections_in_state_vars(
        state: Arc<Mutex<HaxState>>,
        mut new_connection_recv: Receiver<WebSocketProxy>,
    ) {
        debug!("Received new websocket proxy to store in state variable");
        while let Some(mut conn) = new_connection_recv.recv().await {
            match conn.get_server_type() {
                // lobby
                Some(WebSocketServer::LobbyServer) => {
                    let notify_closed = conn.take_notify_closed();

                    let state = state.clone();
                    {
                        let mut locked_state = state.lock().await;
                        if locked_state.lobby_state.is_some() {
                            warn!("lobby socket connection created while one already existed! did it not get cleared correctly?");
                        }
                        locked_state.lobby_state = Some((conn, LobbyState::default()));
                    }

                    match notify_closed {
                        Some(n) => {
                            // create task to clear the socket variable when the connection dies
                            tokio::spawn(async move {
                                // wait for the socket to close
                                n.notified().await;

                                info!("lobby websocket closed");
                                let mut locked_state = state.lock().await;
                                if locked_state.lobby_state.is_none() {
                                    warn!("lobby socket connection was closed but it did not exist yet");
                                }
                                locked_state.lobby_state = None;
                            });
                        }
                        None => warn!("A lobby websocket task was created but no closed Notify was found. Detecting socket closing will not work"),
                    }
                }
                // gameplay
                Some(WebSocketServer::GameServer) => {
                    let notify_closed = conn.take_notify_closed();

                    let state = state.clone();
                    {
                        let mut locked_state = state.lock().await;
                        if locked_state.gameplay_state.is_some() {
                            warn!("gameplay socket connection created while one already existed! did it not get cleared correctly?");
                        }
                        locked_state.gameplay_state = Some((conn, GameplayState::default()));
                    }

                    match notify_closed {
                        Some(n) => {
                            // create task to clear the socket variable when the connection dies
                            tokio::spawn(async move {
                                // wait for the socket to close
                                n.notified().await;

                                info!("gameplay websocket closed");
                                let mut locked_state = state.lock().await;
                                if locked_state.gameplay_state.is_none() {
                                    warn!("gameplay socket connection was closed but it did not exist yet");
                                }
                                locked_state.gameplay_state = None;
                            });
                        }
                        None => warn!("A gameplay websocket task was created but no closed Notify was found. Detecting socket closing will not work"),
                    }
                }
                None => warn!(
                    "WebSocket connection initiated over unknown target port {}",
                    conn.get_port()
                ),
            }
        }

        debug!("websocket proxy receiver closed");
    }
}
