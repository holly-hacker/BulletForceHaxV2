use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use futures_util::lock::Mutex;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use hyper::http::Request;
use hyper::{Body, Response};
use tokio::sync::{mpsc, Notify};
use tokio_tungstenite::tungstenite::Message;
use tower::util::BoxCloneService;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, info_span, trace, warn, Instrument};

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

    pub fn get_server_type(&self) -> Option<WebSocketServer> {
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

pub fn create_service(
    new_connection_sender: mpsc::Sender<WebSocketProxy>,
    shared_state: Arc<Mutex<HaxState>>,
) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
    let service = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .service_fn(move |req| {
            web_socket_proxy_service(req, shared_state.clone(), new_connection_sender.clone())
        });

    BoxCloneService::new(service)
}

#[tracing::instrument(name = "WebSocketProxy", level = "info", skip_all, fields(uri = req.uri().query().unwrap_or("")))]
pub async fn web_socket_proxy_service(
    req: Request<Body>,
    state: Arc<Mutex<HaxState>>,
    new_connection_sender: mpsc::Sender<WebSocketProxy>,
) -> Result<Response<Body>, Infallible> {
    debug!("Incoming websocket request");
    match web_socket_proxy(req, state, new_connection_sender).await {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("Error result while handling proxied request {e:?}");
            Ok(Response::builder()
                .status(500)
                .body(format!("Error while handling request: {e:?}").into())
                .expect("should be able to create basic response"))
        }
    }
}

async fn web_socket_proxy(
    mut incoming_request: Request<Body>,
    shared_state: Arc<Mutex<HaxState>>,
    new_connection_sender: mpsc::Sender<WebSocketProxy>,
) -> anyhow::Result<Response<Body>> {
    // Check if the request is a websocket upgrade request.
    if !hyper_tungstenite::is_upgrade_request(&incoming_request) {
        warn!("Received non-upgrade request in websocket handler");
        return Ok(Response::new(Body::from("This is a websocket endpoint.")));
    }

    let target_uri = {
        let query_string = incoming_request
            .uri()
            .query()
            .ok_or_else(|| anyhow::anyhow!("WebSocket handshake had no query string"))?;
        hyper::Uri::from_str(query_string)
            .with_context(|| "WebSocket handshake query string did not contain valid uri")?
    };
    let target_port = target_uri.port_u16().unwrap_or(0);

    info!("New incoming WebSocket request for {target_uri}");

    let (mut outgoing_response, websocket) =
        hyper_tungstenite::upgrade(&mut incoming_request, None)?;

    let (server_send, server_recv) = {
        // headers to send back to the client
        let response_headers = ["sec-websocket-protocol"];

        // headers to forward to the server
        let forwarded_headers = [
            "sec-websocket-protocol",
            "sec-websocket-key",
            "sec-websocket-version",
            "sec-websocket-extensions",
        ];

        let mut request = Request::builder()
            .method("GET")
            .header("host", target_uri.host().unwrap())
            .header("connection", "Upgrade")
            .header("upgrade", "websocket")
            .uri(&target_uri);
        for header_name in forwarded_headers {
            if let Some(header_value) = incoming_request.headers().get(header_name) {
                request = request.header(header_name, header_value);
            }
        }
        let request = request.body(()).unwrap();
        let (ws_stream, response) = tokio_tungstenite::connect_async(request)
            .await
            .with_context(|| "Failed to connect to server")?;
        debug!("WebSocket handshake has been successfully completed");

        // copy response headers to outging response
        for header_name in response_headers {
            if let Some(header_value) = response.headers().get(header_name) {
                outgoing_response
                    .headers_mut()
                    .insert(header_name, header_value.clone());
            }
        }

        ws_stream.split()
    };

    // we need to make a new task so that we can immediately return the response. hyper-tungstenite
    // wont resolve the `websocket` future.
    tokio::spawn(async move {
        debug!("Awaiting websocket");
        let ws = match websocket.await {
            Ok(ws) => ws,
            Err(e) => {
                error!("Failed to accept websocket connection: {e}");
                return;
            }
        };

        let (client_send, client_recv) = ws.split();
        debug!("Created client streams");

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

        debug!("Sending websocket proxy object over channel");
        let send_result = new_connection_sender
            .send(WebSocketProxy {
                client_send,
                server_send,
                client_to_server,
                server_to_client,
                port: target_uri.port().map(|p| p.as_u16()).unwrap_or(0),
                notify_closed: Some(notify_closed),
            })
            .await;

        if let Err(e) = send_result {
            error!("Failed to send new websocket proxy over channel: {e}");
        };
    });

    Ok(outgoing_response)
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
            debug!("Starting new proxy task");

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
