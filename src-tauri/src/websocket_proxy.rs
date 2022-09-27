use std::collections::HashMap;
use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use futures_util::lock::Mutex;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use hyper::header::HeaderName;
use hyper::http::{HeaderValue, Request};
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
    client_send: Arc<Mutex<WebSocketProxySink>>,
    server_send: Arc<Mutex<WebSocketProxySink>>,

    /// a handle to the task that controls client->server communication
    client_to_server: tokio::task::JoinHandle<()>,
    /// a handle to the task that controls server->client communication
    server_to_client: tokio::task::JoinHandle<()>,
}

#[allow(dead_code)]
impl WebSocketProxy {
    pub async fn send_client(&self, message: Message) -> anyhow::Result<()> {
        self.client_send.lock().await.send_message(message).await?;
        Ok(())
    }

    pub async fn send_server(&self, message: Message) -> anyhow::Result<()> {
        self.server_send.lock().await.send_message(message).await?;
        Ok(())
    }
}

pub struct WebSocketProxySink(SocketSink);

impl WebSocketProxySink {
    pub async fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        self.0.send(message).await?;
        Ok(())
    }
}

struct WebSocketProxyDirection {
    stream: SocketStream,
    sink: Arc<Mutex<WebSocketProxySink>>,
}

impl WebSocketProxyDirection {
    // TODO: I probably want to know which direction this is, for logging reasons
    fn start_task(
        stream: SocketStream,
        sink: Arc<Mutex<WebSocketProxySink>>,
        direction_string: &'static str,
    ) -> tokio::task::JoinHandle<()> {
        let mut direction = WebSocketProxyDirection { stream, sink };
        let span = info_span!("WebSocketProxy", direction = direction_string);
        tokio::spawn(
            async move {
                while let Some(message) = direction.stream.next().await {
                    // TODO: don't unwrap
                    let message = message.unwrap();
                    trace!("Message: {:?}", message);

                    // TODO: install hook here

                    direction
                        .sink
                        .lock()
                        .await
                        .send_message(message)
                        .await
                        .unwrap();
                }

                // TODO: signal death of the connection
            }
            .instrument(span),
        )
    }
}

struct UriCallback {
    tx: tokio::sync::oneshot::Sender<(hyper::Uri, HashMap<HeaderName, HeaderValue>)>,
}

impl Callback for UriCallback {
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

        // headers to forward to the remote server
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

async fn accept_connection(stream: TcpStream) -> WebSocketProxy {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let (tx_request_known, rx_request_known) = oneshot::channel();

    let ws_stream = tokio_tungstenite::accept_hdr_async(
        stream,
        UriCallback {
            tx: tx_request_known,
        },
    )
    .await
    .expect("Error during the websocket handshake occurred");

    // wait for the connection to be established and get the uri from the handshake request
    let (socket_uri, headers_to_forward) = rx_request_known.await.unwrap();
    trace!("Headers to add to server handshake: {headers_to_forward:?}");
    // TODO: don't unwrap here
    let target_uri = hyper::Uri::from_str(socket_uri.query().unwrap()).unwrap();

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
            .expect("Failed to connect to server");
        debug!("WebSocket handshake has been successfully completed");

        ws_stream.split()
    };

    let client_send = Arc::new(Mutex::new(WebSocketProxySink(Box::new(client_send))));
    let server_send = Arc::new(Mutex::new(WebSocketProxySink(Box::new(server_send))));

    let client_to_server =
        WebSocketProxyDirection::start_task(Box::new(client_recv), server_send.clone(), "c->s");
    let server_to_client =
        WebSocketProxyDirection::start_task(Box::new(server_recv), client_send.clone(), "s->c");

    WebSocketProxy {
        client_send,
        server_send,
        client_to_server,
        server_to_client,
    }
}
