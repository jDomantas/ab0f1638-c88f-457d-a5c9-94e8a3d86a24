use result_ext::ResultExt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::ToSocketAddrs;
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use ws;
use resources::ServerResources;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Copy, Clone)]
pub struct ConnectionId(u64);

pub struct Message {
    data: Vec<u8>,
}

impl Message {
    pub fn new(data: Vec<u8>) -> Message {
        Message { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub enum Event {
    Connected {
        id: ConnectionId,
    },
    Disconnected {
        id: ConnectionId,
    },
    Message {
        sender: ConnectionId,
        message: Message,
    },
}

pub struct WebsocketServer {
    inner: Arc<Mutex<InnerServer>>,
    events: Receiver<Event>,
    // We don't have shutdown for now, so join handle is unused.
    #[allow(dead_code)]
    listener_thread: thread::JoinHandle<()>,
}

impl WebsocketServer {
    pub fn listen<A>(resources: Arc<ServerResources>, addr: A) -> WebsocketServer
    where
        A: ToSocketAddrs + Debug + Send + 'static,
    {
        let (event_sender, event_receiver) = mpsc::channel();
        let inner = Arc::new(Mutex::new(InnerServer {
            next_connection_id: ConnectionId(0),
            connections: HashMap::new(),
        }));

        let listener_thread = {
            let inner = inner.clone();
            thread::spawn(move || {
                ws::listen(addr, |ws_sender| {
                    let mut inner_lock = inner.lock().unwrap();
                    let id = inner_lock.generate_id();
                    // This connection might be a request for static files,
                    // so don't emit events or add it to connection list yet.
                    ConnectionHandler {
                        resources: resources.clone(),
                        id,
                        sender: Some(ws_sender),
                        events: event_sender.clone(),
                        inner: inner.clone(),
                    }
                }).log_if_err();
            })
        };

        WebsocketServer {
            inner,
            events: event_receiver,
            listener_thread,
        }
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.events.try_recv().ok()
    }

    pub fn disconnect(&mut self, connection: ConnectionId) {
        info!("disconnecting connection {:?}", connection);
        let mut inner = self.inner.lock().unwrap();
        if let Some(connection) = inner.connections.remove(&connection) {
            connection.close(ws::CloseCode::Protocol).log_if_err();
        } else {
            warn!(
                "tried to disconnect a non-existent connection: {:?}",
                connection
            );
        }
    }

    pub fn send(&mut self, to: ConnectionId, message: Message) {
        let inner = self.inner.lock().unwrap();
        if let Some(connection) = inner.connections.get(&to) {
            let message = ws::Message::Text(String::from_utf8(message.data).unwrap());
            connection.send(message).log_if_err();
        } else {
            warn!(
                "tried to send a message to non-existent connection: {:?}",
                to
            );
        }
    }

    pub fn broadcast(&mut self, message: Message) {
        let inner = self.inner.lock().unwrap();
        // ws-rs is a bit weird at broadcasting: you broadcast by obtaining a
        // random connection, calling `broadcast` on it, and it will send a
        // message to all connections made by the listener that this connection
        // belongs to.
        if let Some(connection) = inner.connections.values().next() {
            let message = ws::Message::Text(String::from_utf8(message.data).unwrap());
            connection.broadcast(message).log_if_err();
        }
    }
}

struct InnerServer {
    next_connection_id: ConnectionId,
    connections: HashMap<ConnectionId, ws::Sender>,
}

impl InnerServer {
    fn generate_id(&mut self) -> ConnectionId {
        let id = self.next_connection_id;
        self.next_connection_id.0 += 1;
        id
    }
}

struct ConnectionHandler {
    resources: Arc<ServerResources>,
    id: ConnectionId,
    sender: Option<ws::Sender>,
    events: Sender<Event>,
    inner: Arc<Mutex<InnerServer>>,
}

impl ws::Handler for ConnectionHandler {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        fn ok(contents: &[u8]) -> ws::Response {
            ws::Response::new(200, "OK", contents.to_vec())
        }

        fn not_found() -> ws::Response {
            ws::Response::new(
                404,
                "Not Found",
                b"404 - Not Found".to_vec(),
            )
        }

        Ok(match req.resource() {
            "/" => ok(&self.resources.index),
            "/bundle.js" => ok(&self.resources.js),
            "/bundle.js.map" => {
                if let Some(source_map) = &self.resources.source_map {
                    ok(source_map)
                } else {
                    not_found()
                }
            }
            "/style.css" => ok(&self.resources.css),
            "/ws" => {
                self.events.send(Event::Connected { id: self.id }).unwrap();
                let sender = self.sender
                    .take()
                    .expect("multiple websocket connection requests on single connection");
                self.inner
                    .lock()
                    .unwrap()
                    .connections
                    .insert(self.id, sender);
                ws::Response::from_request(req)?
            }
            "/game/code.wasm" => ok(&self.resources.package.wasm_module),
            _ => not_found(),
        })
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let data = match msg {
            ws::Message::Text(text) => text.into_bytes(),
            ws::Message::Binary(bytes) => bytes,
        };
        self.events
            .send(Event::Message {
                sender: self.id,
                message: Message::new(data),
            })
            .unwrap();
        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        debug!(
            r#"connection {:?} closing due to "{}" ({:?})"#,
            self.id, reason, code
        );
        let mut inner = self.inner.lock().unwrap();
        inner.connections.remove(&self.id);
        self.events
            .send(Event::Disconnected { id: self.id })
            .unwrap();
    }
}
