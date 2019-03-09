pub mod wasmi;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug};
use std::hash::Hash;

#[derive(Debug)]
pub struct DeserializeError;

pub trait ToBlob {
    fn to_blob(&self) -> Vec<u8>;
}

pub trait Game {
    type World: ToBlob;
    type Input: ToBlob;
    type PlayerId: Eq + Ord + Hash + Copy + Into<u64>;

    fn initial_world(&mut self) -> Self::World;
    fn update_world(&mut self, world: &Self::World) -> Self::World;
    fn update_player(&mut self, world: &Self::World, player: Self::PlayerId, input: &Self::Input) -> Self::World;
    fn add_player(&mut self, world: &Self::World, player: Self::PlayerId) -> Self::World;
    fn remove_player(&mut self, world: &Self::World, player: Self::PlayerId) -> Self::World;
    fn deserialize_input(&mut self, from: &[u8]) -> Result<Self::Input, DeserializeError>;
    fn generate_player_id(&mut self) -> Self::PlayerId;

    fn apply_update(&mut self, world: &Self::World, update: &FrameUpdate<Self>) -> Self::World {
        // FIXME: gross
        let mut removed = update.removed_players.iter();
        let mut world = if let Some(&player) = removed.next() {
            let mut world = self.remove_player(world, player);
            for &player in removed {
                world = self.remove_player(&world, player);
            }
            self.update_world(&world)
        } else {
            self.update_world(world)
        };
        for (&player, input) in &update.player_inputs {
            world = self.update_player(&world, player, input);
        }
        for &player in &update.new_players {
            world = self.add_player(&world, player);
        }
        world
    }
}

pub struct FrameUpdate<G: Game + ?Sized> {
    pub new_players: BTreeSet<G::PlayerId>,
    pub removed_players: BTreeSet<G::PlayerId>,
    pub player_inputs: BTreeMap<G::PlayerId, G::Input>,
}

impl<G: Game + ?Sized> FrameUpdate<G> {
    pub fn new_player(&mut self, player: G::PlayerId) {
        self.new_players.insert(player);
    }

    pub fn input(&mut self, player: G::PlayerId, input: G::Input) {
        self.player_inputs.insert(player, input);
    }

    pub fn remove_player(&mut self, player: G::PlayerId) {
        self.removed_players.insert(player);
    }
}

impl<G: Game + ?Sized> Default for FrameUpdate<G> {
    fn default() -> Self {
        FrameUpdate {
            new_players: Default::default(),
            removed_players: Default::default(),
            player_inputs: Default::default(),
        }
    }
}

impl<G> PartialEq<FrameUpdate<G>> for FrameUpdate<G>
where
    G: Game,
    G::PlayerId: Eq,
    G::Input: Eq,
{
    fn eq(&self, rhs: &Self) -> bool {
        let l = (&self.new_players, &self.removed_players, &self.player_inputs);
        let r = (&rhs.new_players, &rhs.removed_players, &rhs.player_inputs);
        l == r
    }
}

impl<G> Eq for FrameUpdate<G>
where
    G: Game,
    G::PlayerId: Eq,
    G::Input: Eq,
{ }

impl<G> Debug for FrameUpdate<G>
where
    G: Game,
    G::PlayerId: Debug,
    G::Input: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameUpdate")
            .field("new_players", &self.new_players)
            .field("removed_players", &self.removed_players)
            .field("player_inputs", &self.player_inputs)
            .finish()
    }
}
