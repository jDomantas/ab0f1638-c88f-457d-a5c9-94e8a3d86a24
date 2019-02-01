use crate::draw_rectangle;
use crate::game::{Game, Key, KeyboardState, PlayerId, Reader, ReadError, Serialize, Writer};

static COLORS: [u32; 6] = [
    0x0000FF,
    0x00FF00,
    0xFF0000,
    0x00FFFF,
    0xFF00FF,
    0xFFFF00,
];

#[derive(Copy, Clone)]
struct Player {
    id: PlayerId,
    x: i32,
    y: i32,
}

impl Player {
    fn new(id: PlayerId) -> Player {
        Player {
            id,
            x: 10,
            y: 10,
        }
    }

    fn color(&self) -> u32 {
        COLORS[self.id.id() as usize % COLORS.len()]
    }

    fn update(&self, input: Input) -> Player {
        Player {
            id: self.id,
            x: self.x + i32::from(input.dx),
            y: self.y + i32::from(input.dy),
        }
    }
}

impl Serialize for Player {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let (id, x, y) = <(PlayerId, i32, i32)>::read(reader)?;
        Ok(Player { id, x, y })
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (self.id, self.x, self.y).write(writer)
    }
}

#[derive(Copy, Clone)]
pub struct Input {
    dx: i8,
    dy: i8,
}

impl Serialize for Input {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let (dx, dy) = <(i8, i8)>::read(reader)?;
        Ok(Input { dx, dy })
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (self.dx, self.dy).write(writer)
    }
}

#[derive(Default, Clone)]
pub struct World {
    players: Vec<Player>,
}

impl Serialize for World {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let players = Vec::<Player>::read(reader)?;
        Ok(World { players })
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        self.players.write(writer)
    }
}

impl World {
    fn from_players(players: impl Iterator<Item = Player>) -> World {
        World {
            players: players.collect(),
        }
    }

    fn update_players<F: Fn(Player) -> Player>(&self, update: F) -> World {
        World::from_players(self.players().map(update))
    }

    fn add_player(&self, player: Player) -> World {
        let mut world = self.clone();
        world.add_player_in_place(player);
        world
    }

    fn add_player_in_place(&mut self, player: Player) {
        self.players.push(player)
    }

    fn remove_player(&self, id: PlayerId) -> World {
        World::from_players(self.players().filter(|p| p.id != id))
    }

    fn players<'a>(&'a self) -> impl Iterator<Item = Player> + 'a {
        self.players.iter().cloned()
    }
}

#[allow(dead_code)]
pub struct Squares;

impl Game for Squares {
    type World = World;
    type Input = Input;

    fn initial_world() -> Self::World {
        World::default()
    }

    fn update_world(world: &Self::World) -> Self::World {
        world.clone()
    }

    fn update_player(world: &Self::World, id: PlayerId, input: &Self::Input) -> Self::World {
        world.update_players(|player| {
            if player.id == id {
                player.update(*input)
            } else {
                player
            }
        })
    }

    fn add_player(world: &Self::World, player: PlayerId) -> Self::World {
        world.add_player(Player::new(player))
    }

    fn remove_player(world: &Self::World, player: PlayerId) -> Self::World {
        world.remove_player(player)
    }

    fn create_input(keys: KeyboardState) -> Self::Input {
        let (mut dx, mut dy) = (0, 0);
        if keys.is_pressed(Key::Right) { dx += 1; }
        if keys.is_pressed(Key::Left) { dx -= 1; }
        if keys.is_pressed(Key::Down) { dy += 1; }
        if keys.is_pressed(Key::Up) { dy -= 1; }
        Input { dx, dy }
    }

    fn render(world: &Self::World, _local_player: PlayerId, _width: u32, _height: u32) {
        for player in world.players() {
            draw_rectangle(player.x, player.y, 20, 20, player.color());
        }
    }
}
