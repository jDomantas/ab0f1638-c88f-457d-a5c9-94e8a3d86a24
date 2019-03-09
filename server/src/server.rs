use std::collections::{HashMap, VecDeque};
use log::trace;
use crate::game::{FrameUpdate, Game};

pub struct BadJoinError;
pub struct BadInputError;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Copy, Clone)]
pub struct ClientId(u64);

pub struct WorldState<'a, G: Game> {
    pub frame: u64,
    pub world: &'a G::World,
}

pub struct Server<G: Game> {
    game: G,
    frame: u64,
    world: G::World,
    clients: HashMap<ClientId, ClientState<G>>,
    /// Players that need to be removed in the next game tick.
    removed_players: Vec<G::PlayerId>,
    next_client_id: u64,
}

impl<G: Game> Server<G> {
    pub fn new(mut game: G) -> Self {
        let world = game.initial_world();
        Server {
            game,
            frame: 0,
            world,
            clients: HashMap::new(),
            removed_players: Vec::new(),
            next_client_id: 0,
        }
    }

    /// A new client connected to the server. Returned world should be sent to
    /// that client.
    pub fn client_connected(&mut self) -> (ClientId, WorldState<'_, G>) {
        let id = ClientId(self.next_client_id);
        self.next_client_id += 1;
        self.clients.insert(id, ClientState::Connected);
        (id, WorldState { frame: self.frame, world: &self.world })
    }

    /// A client that has already connected wants to join the game. The first
    /// input that the client can send after joining must be for the next frame.
    /// Server might arbitrarily decide that the client cannot join the game
    /// (for example, if client wants to join too far in the past or in the
    /// future). In that case the client should be disconnected.
    pub fn client_joined(&mut self, client: ClientId, on_frame: u64) -> Result<(), BadJoinError> {
        let result = match self.clients.get_mut(&client) {
            None => panic!("client joined without connecting"),
            Some(ClientState::WaitingForJoin { .. }) |
            Some(ClientState::InGame(_)) => {
                // client tried to join multiple times, should be disconnected
                Err(BadJoinError)
            }
            Some(state @ ClientState::Connected) => {
                // currently we only disallow joining in the past
                if on_frame < self.frame {
                    Err(BadJoinError)
                } else {
                    *state = ClientState::WaitingForJoin(WaitingClient {
                        join_frame: on_frame,
                        inputs: InputQueue {
                            next_input_frame: on_frame + 1,
                            inputs: VecDeque::new(),
                        }
                    });
                    Ok(())
                }
            }
        };
        if result.is_err() {
            self.client_disconnected(client);
        }
        result
    }

    /// Client sent an input. Inputs must be sent for each frame without
    /// skipping any, and the first one should be for the next frame after the
    /// one that the client joined on. If those conditions are not met or the
    /// serialized input is not valid, then the client should be disconnected.
    pub fn client_input(&mut self, client: ClientId, frame: u64, serialized: &[u8]) -> Result<(), BadInputError> {
        let result = match self.clients.get_mut(&client) {
            None => panic!("client sent inputs without connecting"),
            Some(ClientState::Connected) => {
                // client tried to send inputs before joining the game,
                // disconnect them
                Err(BadInputError)
            }
            Some(ClientState::WaitingForJoin(WaitingClient { inputs, .. })) |
            Some(ClientState::InGame(InGameClient { inputs, .. })) => {
                self.game
                    .deserialize_input(serialized)
                    .map_err(|_| BadInputError)
                    .and_then(|input| inputs.add_input(frame, input))
            }
        };
        if result.is_err() {
            self.client_disconnected(client);
        }
        result
    }

    /// Advance the game by one frame. Returned frame update should be
    /// broadcasted to all connected clients (including those that haven't
    /// joined the game yet).
    pub fn game_tick(&mut self) -> FrameUpdate<G> {
        let mut update = FrameUpdate::default();
        update.removed_players.extend(self.removed_players.drain(..));
        for client in self.clients.values_mut() {
            match client {
                ClientState::Connected => {}
                ClientState::WaitingForJoin(waiting) => {
                    if waiting.join_frame == self.frame {
                        let player_id = self.game.generate_player_id();
                        update.new_player(player_id);
                        let playing = waiting.into_playing(player_id);
                        *client = ClientState::InGame(playing);
                    }
                }
                ClientState::InGame(client) => {
                    if let Some(input) = client.inputs.get_input(self.frame) {
                        update.input(client.player_id, input);
                    } else {
                        // player hasn't sent inputs for this frame
                        // FIXME: for now we just ignore this, but game
                        // developers might want custom behaviour in this case
                    }
                }
            }
        }
        self.world = self.game.apply_update(&self.world, &update);
        trace!("completed simulation frame #{}", self.frame);
        self.frame += 1;
        update
    }

    /// Client disconnected on its own. This function is idempotent - you can
    /// safely notify the server about a disconnected client multiple times.
    pub fn client_disconnected(&mut self, client: ClientId) {
        match self.clients.remove(&client) {
            None => {}
            Some(ClientState::Connected) |
            Some(ClientState::WaitingForJoin { .. }) => {
                // client is not in-game yet which means that other clients
                // haven't observed them - so we don't need to do anything
            }
            Some(ClientState::InGame(client)) => {
                self.removed_players.push(client.player_id);
            }
        }
    }
}

enum ClientState<G: Game> {
    /// Client has connected but hasn't joined yet.
    Connected,
    /// Client wants to join, but the join frame is still in the future.
    WaitingForJoin(WaitingClient<G>),
    /// Client is in-game.
    InGame(InGameClient<G>),
}

struct WaitingClient<G: Game> {
    join_frame: u64,
    inputs: InputQueue<G>,
}

impl<G: Game> WaitingClient<G> {
    // FIXME: ideally this should take `self` by value, but then one of the
    // places where we use this is difficult to fix :(
    fn into_playing(&mut self, player_id: G::PlayerId) -> InGameClient<G> {
        let temp = InputQueue {
            next_input_frame: 0,
            inputs: VecDeque::new(),
        };
        InGameClient {
            player_id,
            inputs: std::mem::replace(&mut self.inputs, temp),
        }
    }
}

struct InGameClient<G: Game> {
    player_id: G::PlayerId,
    inputs: InputQueue<G>,
}

struct InputQueue<G: Game> {
    next_input_frame: u64,
    inputs: VecDeque<ClientInput<G>>,
}

impl<G: Game> InputQueue<G> {
    fn add_input(&mut self, frame: u64, input: G::Input) -> Result<(), BadInputError> {
        if frame == self.next_input_frame {
            self.inputs.push_back(ClientInput { frame, input });
            self.next_input_frame += 1;
            Ok(())
        } else {
            Err(BadInputError)
        }
    }

    fn get_input(&mut self, frame: u64) -> Option<G::Input> {
        while let Some(input) = self.inputs.pop_front() {
            if input.frame == frame {
                return Some(input.input);
            } else if input.frame > frame {
                self.inputs.push_front(input);
                return None;
            }
        }
        None
    }
}

struct ClientInput<G: Game> {
    frame: u64,
    input: G::Input,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{DeserializeError, ToBlob};

    struct TestGame(u64);

    impl ToBlob for String {
        fn to_blob(&self) -> Vec<u8> { Vec::new() }
    }

    impl ToBlob for Vec<String> {
        fn to_blob(&self) -> Vec<u8> { Vec::new() }
    }

    impl Game for TestGame {
        type World = Vec<String>;
        type Input = String;
        type PlayerId = u64;

        fn initial_world(&mut self) -> Self::World {
            Vec::new()
        }

        fn update_world(&mut self, world: &Self::World) -> Self::World {
            let mut world = world.clone();
            world.push("update".into());
            world
        }

        fn update_player(&mut self, world: &Self::World, player: Self::PlayerId, input: &Self::Input) -> Self::World {
            let mut world = world.clone();
            world.push(format!("input {}: {}", player, input));
            world
        }

        fn add_player(&mut self, world: &Self::World, player: Self::PlayerId) -> Self::World {
            let mut world = world.clone();
            world.push(format!("add {}", player));
            world
        }

        fn remove_player(&mut self, world: &Self::World, player: Self::PlayerId) -> Self::World {
            let mut world = world.clone();
            world.push(format!("remove {}", player));
            world
        }

        fn deserialize_input(&mut self, from: &[u8]) -> Result<Self::Input, DeserializeError> {
            Ok(String::from_utf8(from.to_vec()).expect("bad input"))
        }

        fn generate_player_id(&mut self) -> Self::PlayerId {
            self.0 += 1;
            self.0
        }
    }

    fn server() -> Server<TestGame> {
        Server::new(TestGame(0))
    }

    fn server_with_client() -> (Server<TestGame>, ClientId) {
        let mut server = server();
        let (client, _world) = server.client_connected();
        assert!(server.client_joined(client, 0).is_ok());

        // player is added on next tick
        let tick = server.game_tick();
        let mut expected = FrameUpdate::default();
        expected.new_player(1);
        assert_eq!(tick, expected);

        (server, client)
    }

    #[test]
    fn ticking() {
        let mut server = server();
        // default FrameUpdate because there are no players - so no inputs
        assert_eq!(server.game_tick(), FrameUpdate::default());
        assert_eq!(server.game_tick(), FrameUpdate::default());
        assert_eq!(server.world, vec!["update", "update"]);
        assert_eq!(server.frame, 2);
    }

    #[test]
    fn connect() {
        let mut server = server();
        server.game_tick();
        let (_client, world) = server.client_connected();
        assert_eq!(world.world, &vec!["update"]);
        assert_eq!(world.frame, 1);
    }

    #[test]
    fn connect_and_join() {
        let mut server = server();
        let (client, _world) = server.client_connected();
        assert!(server.client_joined(client, 1).is_ok());
        // default Frame update because it's tick 0
        assert_eq!(server.game_tick(), FrameUpdate::default());
        // frame 1 - a new player should appear
        // test game generates player ids sequentially starting from 1
        let mut expected = FrameUpdate::default();
        expected.new_player(1);
        assert_eq!(server.game_tick(), expected);
    }

    #[test]
    fn joining_twice_is_error() {
        let mut server = server();
        let (client, _world) = server.client_connected();
        assert!(server.client_joined(client, 1).is_ok());
        assert!(server.client_joined(client, 1).is_err());
    }

    #[test]
    fn client_input() {
        let mut server = server();
        let (client, _world) = server.client_connected();
        assert!(server.client_joined(client, 0).is_ok());

        // player is added on next tick
        let tick = server.game_tick();
        let mut expected = FrameUpdate::default();
        expected.new_player(1);
        assert_eq!(tick, expected);

        assert!(server.client_input(client, 1, "abc".as_bytes()).is_ok());

        let tick = server.game_tick();
        let mut expected = FrameUpdate::default();
        expected.input(1, "abc".to_string());
        assert_eq!(tick, expected);
    }

    #[test]
    fn connect_join_leave() {
        let (mut server, client) = server_with_client();

        server.client_disconnected(client);

        let tick = server.game_tick();
        let mut expected = FrameUpdate::default();
        expected.remove_player(1);
        assert_eq!(tick, expected);
    }

    #[test]
    fn input_skip() {
        let (mut server, _client) = server_with_client();

        // client does not send any inputs, so no inputs in update
        assert_eq!(server.game_tick(), FrameUpdate::default());
        assert_eq!(server.game_tick(), FrameUpdate::default());
        assert_eq!(server.game_tick(), FrameUpdate::default());
    }
}
