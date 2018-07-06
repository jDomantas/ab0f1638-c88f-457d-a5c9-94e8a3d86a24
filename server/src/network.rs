use result_ext::ResultExt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::{self, Read};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use ws;

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
    pub fn listen<A>(addr: A) -> WebsocketServer
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
    id: ConnectionId,
    sender: Option<ws::Sender>,
    events: Sender<Event>,
    inner: Arc<Mutex<InnerServer>>,
}

impl ws::Handler for ConnectionHandler {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        const INDEX_PATH: &str = "../client/target/index.html";
        const JS_PATH: &str = "../client/target/bundle.js";
        const SOURCE_MAP_PATH: &str = "../client/target/bundle.js.map";
        const CSS_PATH: &str = "../client/target/style.css";

        fn ok<P: AsRef<Path>>(path: P) -> io::Result<ws::Response> {
            let mut file = fs::File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(ws::Response::new(200, "OK", data))
        }

        match req.resource() {
            "/" => Ok(ok(INDEX_PATH)?),
            "/bundle.js" => Ok(ok(JS_PATH)?),
            "/bundle.js.map" => Ok(ok(SOURCE_MAP_PATH)?),
            "/style.css" => Ok(ok(CSS_PATH)?),
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
                ws::Response::from_request(req)
            }
            _ => Ok(ws::Response::new(
                404,
                "Not Found",
                b"404 - Not Found".to_vec(),
            )),
        }
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
