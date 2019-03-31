use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;
use log::trace;
use crate::server::{Server, ClientId};
use crate::game::{Game, ToBlob};
use crate::network::{ConnectionId, Event, Message, WebsocketServer};
use crate::protocol;

pub struct GameLoop<G: Game> {
    network_server: WebsocketServer,
    game_server: Server<G>,
    clients: HashMap<ConnectionId, ClientId>,
}

impl<G: Game> GameLoop<G> {
    pub fn new(network_server: WebsocketServer, game_server: Server<G>) -> Self {
        GameLoop {
            network_server,
            game_server,
            clients: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut last_frame_time = Instant::now();
        let frames_per_second = 60;
        let frame_time = Duration::from_micros(1_000_000 / frames_per_second);

        loop {
            self.process_network_events();
            let current_time = Instant::now();
            let next_frame_time = last_frame_time + frame_time;
            if next_frame_time > current_time {
                thread::sleep(next_frame_time - current_time);
            } else {
                self.game_tick();
                last_frame_time += frame_time;
            }
        }
    }
    
    fn process_network_events(&mut self) {
        while let Some(event) = self.network_server.poll_event() {
            match event {
                Event::Message { sender, message } => {
                    self.received_message(sender, message);
                }
                Event::Connected { id } => {
                    self.client_connected(id);
                }
                Event::Disconnected { id } => {
                    self.disconnect_client(id);
                }
            }
        }
    }
    
    fn client_connected(&mut self, connection: ConnectionId) {
        let (client, world) = self.game_server.client_connected();
        self.clients.insert(connection, client);
        let world = protocol::World {
            frame: world.frame,
            local_player_id: world.local_player_id,
            world: world.world.to_blob(),
        };
        let message = Message::new(protocol::world_to_json(&world).into_bytes());
        self.network_server.send(connection, message);
    }

    fn disconnect_client(&mut self, connection: ConnectionId) {
        if let Some(client) = self.clients.remove(&connection) {
            self.game_server.client_disconnected(client);
            self.network_server.disconnect(connection);
        }
    }

    fn received_message(&mut self, sender: ConnectionId, message: Message) {
        // FIXME: hack, json messages are being passed through as binary blobs
        let message = std::str::from_utf8(message.data()).expect("invalid utf-8");
        let message = match protocol::message_from_json(message) {
            Ok(msg) => msg,
            Err(_) => {
                trace!(
                    "client {:?} sent malformed message, disconnecting",
                    sender,
                );
                self.disconnect_client(sender);
                return;
            }
        };
        
        let client = self.clients[&sender];
        let is_ok = match message {
            protocol::ClientMessage::Join { frame } => {
                self.game_server.client_joined(client, frame).is_ok()
            }
            protocol::ClientMessage::Input { frame, input } => {
                self.game_server.client_input(client, frame, &input).is_ok()
            }
        };

        if !is_ok {
            self.disconnect_client(sender);
        }
    }

    fn game_tick(&mut self) {
        let update = self.game_server.game_tick();
        let update = protocol::Update {
            new_players: update
                .new_players
                .into_iter()
                .map(|p| p.into())
                .collect(),
            removed_players: update
                .removed_players
                .into_iter()
                .map(|p| p.into())
                .collect(),
            inputs: update
                .player_inputs
                .into_iter()
                .map(|(p, i)| (p.into(), i.to_blob()))
                .collect(),
        };
        let message = Message::new(protocol::update_to_json(&update).into_bytes());
        self.network_server.broadcast(message);
    }
}
