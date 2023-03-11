use std::{cell::RefCell, rc::Rc};

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use log::{info, trace};
use shared::HaxStateNetwork;
use yew::{platform::spawn_local, Callback};

pub struct HaxIpc {
    write_queue: Rc<RefCell<Vec<Message>>>,
}

impl HaxIpc {
    pub fn new_connect(callback: Callback<HaxStateNetwork>) -> Self {
        let window = web_sys::window().expect("get window object");
        let host = window.location().host().expect("get origin");
        let url = format!("ws://{host}/hax/ws");

        info!("Connecting to websocket at {url}");
        let ws = WebSocket::open(&url).unwrap();
        let (mut write, mut read) = ws.split();

        // handle incoming messages
        spawn_local(async move {
            while let Some(msg) = read.next().await {
                trace!("WS message: {msg:?}");
                if let Ok(Message::Bytes(bytes)) = msg {
                    let parsed: HaxStateNetwork =
                        postcard::from_bytes(&bytes).expect("parse incoming data");
                    callback.emit(parsed);
                };
            }
            info!("WebSocket Closed")
        });

        // handle outgoing messages
        let write_queue = Rc::new(RefCell::new(Vec::new()));
        let write_queue_cloned = write_queue.clone();
        spawn_local(async move {
            // TODO: some signal to close this task?
            loop {
                // swap the stored list with an empty one
                // this is pretty fast since new vecs have no heap allocs
                let vec = write_queue_cloned.replace(vec![]);
                if !vec.is_empty() {
                    for msg in vec {
                        write.feed(msg).await.expect("feed message");
                    }
                    write.flush().await.expect("flush message");
                }
                TimeoutFuture::new(100).await;
            }
        });

        Self { write_queue }
    }

    pub fn send(&self, data: String) {
        self.write_queue.borrow_mut().push(Message::Text(data));
    }
}
