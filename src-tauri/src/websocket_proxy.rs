use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

use futures_util::lock::Mutex;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::{handshake::server::Callback, Message};

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
    ) -> tokio::task::JoinHandle<()> {
        let mut direction = WebSocketProxyDirection { stream, sink };
        tokio::spawn(async move {
            while let Some(message) = direction.stream.next().await {
                // TODO: don't unwrap
                let message = message.unwrap();
                println!("Message: {:?}", message);

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
        })
    }
}

struct UriCallback {
    tx: tokio::sync::oneshot::Sender<hyper::Uri>,
}

impl Callback for UriCallback {
    fn on_request(
        self,
        request: &tokio_tungstenite::tungstenite::handshake::server::Request,
        response: tokio_tungstenite::tungstenite::handshake::server::Response,
    ) -> Result<
        tokio_tungstenite::tungstenite::handshake::server::Response,
        tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
    > {
        let uri = request.uri().clone();
        self.tx.send(uri).unwrap();
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
    println!("Peer address: {}", addr);

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
    let socket_uri = rx_request_known.await.unwrap();
    // TODO: don't unwrap here
    let target_uri = hyper::Uri::from_str(socket_uri.query().unwrap()).unwrap();

    println!("New WebSocket connection: {}", addr);

    let (client_send, client_recv) = ws_stream.split();

    let (server_send, server_recv) = {
        let (ws_stream, _) = tokio_tungstenite::connect_async(target_uri)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        ws_stream.split()
    };

    let client_send = Arc::new(Mutex::new(WebSocketProxySink(Box::new(client_send))));
    let server_send = Arc::new(Mutex::new(WebSocketProxySink(Box::new(server_send))));

    let client_to_server =
        WebSocketProxyDirection::start_task(Box::new(client_recv), server_send.clone());
    let server_to_client =
        WebSocketProxyDirection::start_task(Box::new(server_recv), client_send.clone());

    WebSocketProxy {
        client_send,
        server_send,
        client_to_server,
        server_to_client,
    }
}
