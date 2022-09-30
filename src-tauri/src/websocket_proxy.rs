use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use anyhow::{Context, Result};
use futures_util::lock::Mutex;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use hyper::header::HeaderName;
use hyper::http::{HeaderValue, Request};
use photon_lib::photon_message::PhotonMessage;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::{handshake::server::Callback, Message};
use tracing::{debug, info, info_span, trace, Instrument};

type SocketStream =
    Box<dyn Stream<Item = tokio_tungstenite::tungstenite::Result<Message>> + Unpin + Send>;
type SocketSink =
    Box<dyn Sink<Message, Error = tokio_tungstenite::tungstenite::error::Error> + Unpin + Send>;

#[allow(dead_code)]
/// A struct holding a conceptual websocket proxy connection
struct WebSocketProxy {
    client_send: Arc<Mutex<SocketSink>>,
    server_send: Arc<Mutex<SocketSink>>,

    /// a handle to the task that controls client->server communication
    client_to_server: tokio::task::JoinHandle<()>,
    /// a handle to the task that controls server->client communication
    server_to_client: tokio::task::JoinHandle<()>,
}

#[allow(dead_code)]
impl WebSocketProxy {
    pub async fn send_client(&self, message: Message) -> anyhow::Result<()> {
        self.client_send.lock().await.send(message).await?;
        Ok(())
    }

    pub async fn send_server(&self, message: Message) -> anyhow::Result<()> {
        self.server_send.lock().await.send(message).await?;
        Ok(())
    }
}

/// Starts listening for client socket connections
pub async fn block_on_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 48898));

    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _socket_address)) = listener.accept().await {
        tokio::spawn(async move {
            let _proxy = accept_connection(stream).await;
            // TODO: send this proxy over a channel so it can be installed in a global object
        });
    }
}

async fn accept_connection(stream: TcpStream) -> Result<WebSocketProxy> {
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

    // these explicit type definitions are required because it tells the compiler to use `Box<impl SomeTrait>`
    let client_send: SocketSink = Box::new(client_send);
    let server_send: SocketSink = Box::new(server_send);

    let client_send = Arc::new(Mutex::new(client_send));
    let server_send = Arc::new(Mutex::new(server_send));

    let client_to_server = start_proxy_task(Box::new(client_recv), server_send.clone(), "c->s");
    let server_to_client = start_proxy_task(Box::new(server_recv), client_send.clone(), "s->c");

    Ok(WebSocketProxy {
        client_send,
        server_send,
        client_to_server,
        server_to_client,
    })
}

fn start_proxy_task(
    mut stream: SocketStream,
    sink: Arc<Mutex<SocketSink>>,
    direction: &'static str,
) -> tokio::task::JoinHandle<()> {
    let span = info_span!("WebSocketProxy", direction);
    tokio::spawn(
        async move {
            while let Some(message) = stream.next().await {
                // TODO: don't unwrap
                let message = message.unwrap();
                trace!("Message: {:?}", message);

                // TODO: install proper hook
                if let Message::Binary(b) = &message {
                    test_hook(b, direction);
                }

                sink.lock().await.send(message).await.unwrap();
            }

            // TODO: signal death of the connection
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

fn test_hook(data: &[u8], direction: &'static str) {
    let mut bytes = bytes::Bytes::copy_from_slice(data);
    let deserialized = PhotonMessage::from_websocket_bytes(&mut bytes);
    debug!("{direction} data: {deserialized:?}");
}
