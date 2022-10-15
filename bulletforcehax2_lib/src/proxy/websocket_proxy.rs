use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use anyhow::{Context, Result};
use futures_util::lock::Mutex;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use hyper::header::HeaderName;
use hyper::http::{HeaderValue, Request};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, Notify};
use tokio_tungstenite::tungstenite::{handshake::server::Callback, Message};
use tracing::{debug, error, info, info_span, trace, Instrument};

use super::{Direction, WebSocketServer};
use crate::hax::HaxState;

type SocketStream =
    Box<dyn Stream<Item = tokio_tungstenite::tungstenite::Result<Message>> + Unpin + Send>;
type SocketSink =
    Box<dyn Sink<Message, Error = tokio_tungstenite::tungstenite::error::Error> + Unpin + Send>;

/// A struct holding a conceptual websocket proxy connection
pub struct WebSocketProxy {
    /// a sink to allow sending messages to the client at arbitrary times
    client_send: Arc<Mutex<SocketSink>>,
    /// a sink to allow sending messages to the server at arbitrary times
    server_send: Arc<Mutex<SocketSink>>,

    /// a handle to the task that controls client->server communication
    #[allow(dead_code)]
    client_to_server: tokio::task::JoinHandle<()>,
    /// a handle to the task that controls server->client communication
    #[allow(dead_code)]
    server_to_client: tokio::task::JoinHandle<()>,

    port: u16,

    notify_closed: Option<Arc<Notify>>,
}

impl WebSocketProxy {
    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_server(&self) -> Option<WebSocketServer> {
        WebSocketServer::from_port(self.port)
    }

    pub(crate) fn take_notify_closed(&mut self) -> Option<Arc<Notify>> {
        self.notify_closed.take()
    }

    #[allow(dead_code)]
    pub async fn send_client(&self, message: Message) -> anyhow::Result<()> {
        self.client_send.lock().await.send(message).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send_server(&self, message: Message) -> anyhow::Result<()> {
        self.server_send.lock().await.send(message).await?;
        Ok(())
    }
}

/// Starts listening for client socket connections
pub async fn block_on_server(
    new_connection_sender: mpsc::Sender<WebSocketProxy>,
    shared_state: Arc<Mutex<HaxState>>,
) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 48899));

    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _socket_address)) = listener.accept().await {
        let sender = new_connection_sender.clone();
        let state = shared_state.clone();
        tokio::spawn(async move {
            let result = accept_connection(stream, state).await;
            match result {
                Ok(connection) => {
                    info!(
                        "new websocket connection accepted for target port {}",
                        connection.port
                    );
                    if sender.send(connection).await.is_err() {
                        error!("Tried to send websocket connection over channel but it failed");
                    }
                }
                Err(e) => {
                    error!("Error while accepting new websocket connection: {e}");
                }
            }
        });
    }
}

async fn accept_connection(
    stream: TcpStream,
    shared_state: Arc<Mutex<HaxState>>,
) -> Result<WebSocketProxy> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let (tx_handshake, rx_handshake) = oneshot::channel();

    let ws_stream = tokio_tungstenite::accept_hdr_async(
        stream,
        WebsocketHandshakeCallback { tx: tx_handshake },
    )
    .await
    .with_context(|| "Error during the websocket handshake occurred")?;

    // wait for the connection to be established and get the uri from the handshake request
    let (socket_uri, headers_to_forward) = rx_handshake.await.unwrap();
    trace!("Headers to add to server handshake: {headers_to_forward:?}");

    let target_uri = {
        let query_string = socket_uri
            .query()
            .ok_or_else(|| anyhow::anyhow!("WebSocket handshake had no query string"))?;
        hyper::Uri::from_str(query_string)
            .with_context(|| "WebSocket handshake query string did not contain valid uri")?
    };
    let target_port = target_uri.port_u16().unwrap_or(0);

    info!("New WebSocket connection from {addr} for uri {target_uri}");

    let (client_send, client_recv) = ws_stream.split();

    let (server_send, server_recv) = {
        let mut request = Request::builder()
            .method("GET")
            .header("host", target_uri.host().unwrap())
            .header("connection", "Upgrade")
            .header("upgrade", "websocket")
            .uri(&target_uri);
        for (header_name, header_value) in headers_to_forward {
            request = request.header(header_name, header_value);
        }
        let request = request.body(()).unwrap();
        let (ws_stream, _) = tokio_tungstenite::connect_async(request)
            .await
            .with_context(|| "Failed to connect to server")?;
        debug!("WebSocket handshake has been successfully completed");

        ws_stream.split()
    };

    let notify_closed = Arc::new(Notify::new());

    // these explicit type definitions are required because it tells the compiler to use `Box<impl SomeTrait>`
    let client_send: SocketSink = Box::new(client_send);
    let server_send: SocketSink = Box::new(server_send);

    let client_send = Arc::new(Mutex::new(client_send));
    let server_send = Arc::new(Mutex::new(server_send));

    let client_to_server = start_proxy_task(
        Box::new(client_recv),
        server_send.clone(),
        target_port,
        Direction::ClientToServer,
        notify_closed.clone(),
        shared_state.clone(),
    );
    let server_to_client = start_proxy_task(
        Box::new(server_recv),
        client_send.clone(),
        target_port,
        Direction::ServerToClient,
        notify_closed.clone(),
        shared_state.clone(),
    );

    Ok(WebSocketProxy {
        client_send,
        server_send,
        client_to_server,
        server_to_client,
        port: target_uri.port().map(|p| p.as_u16()).unwrap_or(0),
        notify_closed: Some(notify_closed),
    })
}

fn start_proxy_task(
    mut stream: SocketStream,
    sink: Arc<Mutex<SocketSink>>,
    server_port: u16,
    direction: Direction,
    notify_closed: Arc<Notify>,
    shared_state: Arc<Mutex<HaxState>>,
) -> tokio::task::JoinHandle<()> {
    let server = WebSocketServer::from_port(server_port);
    let span = match server {
        Some(server) => info_span!("WebSocketProxy", "{} {}", server, direction),
        None => info_span!("WebSocketProxy", "port:{} {}", server_port, direction),
    };
    tokio::spawn(
        async move {
            while let Some(message) = stream.next().await {
                // TODO: don't unwrap. what can this error on?
                let mut message = message.unwrap();
                trace!("Message: {:?}", message);

                // handle hook
                if let Some(server) = server {
                    // TODO: install proper hook
                    if let Message::Binary(bytes) = &mut message {
                        let result = HaxState::websocket_hook(
                            shared_state.clone(),
                            bytes,
                            server,
                            direction,
                        );
                        match result {
                            Ok(true) => (),        // message should be forwarded
                            Ok(false) => continue, // message should not be sent
                            Err(e) => {
                                error!("Error during websocket hook handler: {}", e);
                            }
                        }
                    }
                }

                let send_result = sink.lock().await.send(message).await;

                match send_result {
                    Ok(_) => (),
                    Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed) => (), // this is fine
                    Err(e) => error!("An error occured while sending a packet: {e}"),
                }
            }

            // signal death of the connection
            notify_closed.notify_one();
        }
        .instrument(span),
    )
}

struct WebsocketHandshakeCallback {
    tx: tokio::sync::oneshot::Sender<(hyper::Uri, Vec<(HeaderName, HeaderValue)>)>,
}

impl Callback for WebsocketHandshakeCallback {
    fn on_request(
        self,
        request: &tokio_tungstenite::tungstenite::handshake::server::Request,
        mut response: tokio_tungstenite::tungstenite::handshake::server::Response,
    ) -> Result<
        tokio_tungstenite::tungstenite::handshake::server::Response,
        tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
    > {
        // headers to send back to the client
        let response_headers = ["sec-websocket-protocol"];

        // headers to forward to the server
        let forwarded_headers = [
            "sec-websocket-protocol",
            "sec-websocket-key",
            "sec-websocket-version",
            "sec-websocket-extensions",
        ];

        let map = request
            .headers()
            .iter()
            .filter(|(name, _)| forwarded_headers.into_iter().any(|h| *name == h))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();

        let uri = request.uri().clone();
        self.tx.send((uri, map)).unwrap();

        trace!("Websocket client headers: {:?}", request.headers());

        for header in response_headers.into_iter() {
            if let Some(header_value) = request.headers().get(header) {
                // technically not correct because there can be multiple protocols
                response.headers_mut().insert(header, header_value.clone());
            }
        }
        Ok(response)
    }
}
