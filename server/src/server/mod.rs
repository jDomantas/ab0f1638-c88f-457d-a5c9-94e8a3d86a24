mod client;

use self::client::Client;
use game::{Game, Input, PlayerId, World};
use network::{ConnectionId, Event, Message, WebsocketServer};
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::str;
use std::thread;
use std::time::{Duration, Instant};

pub struct Server {
    network_server: WebsocketServer,
    clients: HashMap<ConnectionId, Client>,
    future_inputs: HashMap<u64, InputSet>,
    frame: u64,
    game: Game,
    world: World,
    next_update_info: UpdateInfo,
}

#[derive(Default)]
struct InputSet {
    inputs: BTreeMap<PlayerId, Input>,
}

struct UpdateInfo {
    frame: u64,
    new_players: Vec<PlayerId>,
    removed_players: Vec<PlayerId>,
    inputs: InputSet,
}

impl UpdateInfo {
    fn reset(&mut self, frame: u64) {
        self.frame = frame;
        self.new_players.clear();
        self.removed_players.clear();
        self.inputs.inputs.clear();
    }
}

impl Server {
    pub fn new(network_server: WebsocketServer, game: Game, world: World) -> Server {
        Server {
            network_server,
            clients: HashMap::new(),
            future_inputs: HashMap::new(),
            frame: 0,
            game,
            world,
            next_update_info: UpdateInfo {
                frame: 1,
                new_players: Vec::new(),
                removed_players: Vec::new(),
                inputs: InputSet {
                    inputs: BTreeMap::new(),
                },
            },
        }
    }

    pub fn run(&mut self) {
        let mut last_frame_time = Instant::now();
        let frames_per_second = 60;
        let frame_time = Duration::from_nanos(1_000_000_000 / frames_per_second);

        loop {
            self.process_network_events();
            let current_time = Instant::now();
            let next_frame_time = last_frame_time + frame_time;
            if next_frame_time > current_time {
                thread::sleep(next_frame_time - current_time);
            } else {
                self.try_run_frame();
                last_frame_time += frame_time;
            }
        }
    }

    fn process_network_events(&mut self) {
        loop {
            let event = match self.network_server.poll_event() {
                Some(event) => event,
                None => return,
            };
            match event {
                Event::Message { sender, message } => {
                    self.received_message(sender, message);
                }
                Event::Connected { id } => {
                    self.player_connected(id);
                }
                Event::Disconnected { id } => {
                    self.disconnect_client(id);
                }
            }
        }
    }

    fn generate_player_id(&mut self) -> PlayerId {
        self.game.generate_player_id()
    }

    fn player_connected(&mut self, connection: ConnectionId) {
        let player_id = self.generate_player_id();
        self.next_update_info.new_players.push(player_id.clone());
        self.clients
            .insert(connection, Client::new(player_id, self.frame));
        self.send_world_state(connection);
    }

    fn received_message(&mut self, sender: ConnectionId, message: Message) {
        trace!("received message from {:?}", sender);
        let message = match deserialize_message(message.data()) {
            Some(message) => message,
            None => {
                trace!("message from {:?} is malformed, disconnecting", sender);
                self.disconnect_client(sender);
                return;
            }
        };

        if self.clients
            .get_mut(&sender)
            .expect("got message from non-existent client")
            .received_input(message.frame)
            .is_err()
        {
            trace!(
                "message from {:?} is tagged with wrong frame, disconnecting",
                sender
            );
            self.disconnect_client(sender);
            return;
        }

        if message.frame <= self.frame {
            // Currently we are pausing the simulation if we don't have all the
            // needed inputs and we just checked that client sent input for
            // correct frame.
            unreachable!("received old input");
        }

        let input = match self.game.deserialize_input(message.input.as_bytes()) {
            Some(input) => input,
            None => {
                trace!(
                    "message from {:?} contains malformed input, disconnecting",
                    sender
                );
                self.disconnect_client(sender);
                return;
            }
        };

        self.future_inputs
            .entry(message.frame)
            .or_insert_with(Default::default)
            .inputs
            .insert(self.clients[&sender].player_id().clone(), input);
    }

    fn try_run_frame(&mut self) {
        let next_frame = self.frame + 1;

        let inputs_available = self.future_inputs
            .get(&next_frame)
            .map(|set| set.inputs.len())
            .unwrap_or(0);

        if inputs_available < self.clients.len() {
            trace!(
                "cannot simulate frame, only got {}/{} inputs",
                inputs_available,
                self.clients.len(),
            );
            return;
        }

        for player in &self.next_update_info.removed_players {
            self.world = self.game.remove_player(player, &self.world);
        }
        self.world = self.game.update_world(&self.world);

        if let Some(set) = self.future_inputs.remove(&next_frame) {
            for (player, input) in &set.inputs {
                self.world = self.game.update_player(player, input, &self.world);
            }
            self.next_update_info.inputs = set;
        }

        for player in &self.next_update_info.new_players {
            self.world = self.game.add_player(player, &self.world);
        }

        self.frame = next_frame;
        self.broadcast_update();
        self.next_update_info.reset(next_frame);

        debug!("completed to simulation frame {}", next_frame);
    }

    fn broadcast_update(&mut self) {
        let json = self.build_update_json();
        self.network_server
            .broadcast(Message::new(json.into_bytes()));
    }

    fn send_world_state(&mut self, to: ConnectionId) {
        let json = self.build_world_json(to);
        self.network_server
            .send(to, Message::new(json.into_bytes()));
    }

    fn disconnect_client(&mut self, id: ConnectionId) {
        trace!("disconnecting {:?}", id);

        // This gets called when socket is closed, even if we closed it from
        // this side. In that case we have already removed the client when we
        // disconnected him - so if we don't have that client, just return.
        let client = match self.clients.remove(&id) {
            Some(client) => client,
            None => return,
        };

        self.network_server.disconnect(id);

        for set in self.future_inputs.values_mut() {
            set.inputs.remove(client.player_id());
        }

        // If this player connected on this frame, then he didn't yet do
        // anything and we can simply forget that he ever connected. Otherwise,
        // remember the disconnect event for next update.
        if let Some(index) = self.next_update_info
            .new_players
            .iter()
            .position(|c| c == client.player_id())
        {
            self.next_update_info.new_players.swap_remove(index);
        } else {
            self.next_update_info
                .removed_players
                .push(client.player_id().clone());
        }
    }

    fn build_world_json(&mut self, client: ConnectionId) -> String {
        #[derive(Serialize)]
        struct Data<'a> {
            #[serde(rename = "localPlayer")]
            local_player: u64,
            frame: u64,
            world: &'a str,
        }
        let player_id = self.clients[&client].player_id().to_u64();
        let world = self.game.serialize_world(&self.world);
        let world_bytes = self.game.buffer_data(&world);
        let data = Data {
            local_player: player_id,
            frame: self.frame,
            // Temporary hack: currently we are sending dummy values anyways.
            // Eventually we will probably need to send binary messages instead of json.
            world: str::from_utf8(world_bytes).unwrap(),
        };
        serde_json::to_string(&data).expect("world state serialization failed")
    }

    fn build_update_json(&mut self) -> String {
        #[derive(Serialize, Default)]
        struct Data {
            frame: u64,
            #[serde(rename = "newPlayers")]
            new_players: Vec<u64>,
            #[serde(rename = "removedPlayers")]
            removed_players: Vec<u64>,
            inputs: HashMap<String, String>,
        }
        let game = &mut self.game;
        let data = Data {
            frame: self.frame,
            new_players: self.next_update_info
                .new_players
                .iter()
                .map(PlayerId::to_u64)
                .collect(),
            removed_players: self.next_update_info
                .removed_players
                .iter()
                .map(PlayerId::to_u64)
                .collect(),
            inputs: self.next_update_info
                .inputs
                .inputs
                .iter()
                .map(|(id, input)| {
                    let input = game.serialize_input(input);
                    let input_bytes = game.buffer_data(&input);
                    // Same hack as with world state serialization: for now
                    // pretending that bytes and strings are the same thing.
                    let input_string = str::from_utf8(input_bytes).unwrap().to_string();
                    (id.to_u64().to_string(), input_string)
                })
                .collect(),
        };
        serde_json::to_string(&data).expect("frame input serialization failed")
    }
}

#[derive(Deserialize)]
struct InputMessage {
    frame: u64,
    input: String,
}

fn deserialize_message(binary: &[u8]) -> Option<InputMessage> {
    serde_json::from_reader(binary).ok()
}
