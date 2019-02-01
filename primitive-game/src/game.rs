#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct PlayerId {
    id: u32,
}

impl PlayerId {
    pub fn new(id: u32) -> Self {
        PlayerId { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

pub trait Reader {
    fn read_byte(&mut self) -> Result<u8, ReadError>;
}

impl<'a> Reader for &'a [u8] {
    fn read_byte(&mut self) -> Result<u8, ReadError> {
        if self.is_empty() {
            Err(ReadError)
        } else {
            let byte = self[0];
            *self = &self[1..];
            Ok(byte)
        }
    }
}

pub trait Writer {
    fn write_byte(&mut self, byte: u8);
}

impl Writer for Vec<u8> {
    fn write_byte(&mut self, byte: u8) {
        self.push(byte);
    }
}

#[derive(Debug)]
pub struct ReadError;

pub trait Serialize: Sized {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError>;
    fn write<W: Writer>(&self, writer: &mut W);
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Key {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
    J = 9,
    K = 10,
    L = 11,
    M = 12,
    N = 13,
    O = 14,
    P = 15,
    Q = 16,
    R = 17,
    S = 18,
    T = 19,
    U = 20,
    V = 21,
    W = 22,
    X = 23,
    Y = 24,
    Z = 25,
    Up = 32,
    Down = 33,
    Left = 34,
    Right = 35,
    Num0 = 36,
    Num1 = 37,
    Num2 = 38,
    Num3 = 39,
    Num4 = 40,
    Num5 = 41,
    Num6 = 42,
    Num7 = 43,
    Num8 = 44,
    Num9 = 45,
}

#[derive(Debug, Copy, Clone)]
pub struct KeyboardState {
    old_keys: u64,
    keys: u64,
}

impl KeyboardState {
    pub fn new(letters: u32, old_letters: u32, other: u32, old_other: u32) -> Self {
        let keys = (u64::from(other) << 32) + u64::from(letters);
        let old_keys = (u64::from(old_other) << 32) + u64::from(old_letters);
        KeyboardState { keys, old_keys }
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self, key: Key) -> bool {
        let key = key as u8;
        (self.keys >> key) & 1 != 0
    }

    #[allow(dead_code)]
    pub fn was_pressed(&self, key: Key) -> bool {
        let key = key as u8;
        (self.old_keys >> key) & 1 != 0
    }

    #[allow(dead_code)]
    pub fn is_just_pressed(&self, key: Key) -> bool {
        self.is_pressed(key) && !self.was_pressed(key)
    }
}

pub trait Game {
    type World: Serialize;
    type Input: Serialize;

    fn initial_world() -> Self::World;
    fn update_world(world: &Self::World) -> Self::World;
    fn update_player(world: &Self::World, player: PlayerId, input: &Self::Input) -> Self::World;
    fn add_player(world: &Self::World, player: PlayerId) -> Self::World;
    fn remove_player(world: &Self::World, player: PlayerId) -> Self::World;
    fn create_input(keys: KeyboardState) -> Self::Input;
    fn render(world: &Self::World, local_player: PlayerId, width: u32, height: u32);
}

impl Serialize for u8 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        reader.read_byte()
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        writer.write_byte(*self);
    }
}

impl Serialize for u16 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let low = u16::from(reader.read_byte()?);
        let high = u16::from(reader.read_byte()?);
        Ok(low + high * 256)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        writer.write_byte(*self as u8);
        writer.write_byte((*self / 256) as u8);
    }
}

impl Serialize for u32 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let low = u32::from(u16::read(reader)?);
        let high = u32::from(u16::read(reader)?);
        Ok(low + (high << 16))
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u16).write(writer);
        ((*self >> 16) as u16).write(writer);
    }
}

impl Serialize for u64 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let low = u64::from(u32::read(reader)?);
        let high = u64::from(u32::read(reader)?);
        Ok(low + (high << 32))
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u32).write(writer);
        ((*self >> 32) as u32).write(writer);
    }
}

impl Serialize for i8 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        u8::read(reader).map(|x| x as i8)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u8).write(writer)
    }
}

impl Serialize for i16 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        u16::read(reader).map(|x| x as i16)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u16).write(writer)
    }
}

impl Serialize for i32 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        u32::read(reader).map(|x| x as i32)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u32).write(writer)
    }
}

impl Serialize for i64 {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        u64::read(reader).map(|x| x as i64)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (*self as u64).write(writer)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let size = u32::read(reader)? as usize;
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(T::read(reader)?);
        }
        Ok(v)
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        (self.len() as u32).write(writer);
        for item in self {
            item.write(writer);
        }
    }
}

impl Serialize for PlayerId {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        Ok(PlayerId { id: u32::read(reader)? })
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        self.id.write(writer);
    }
}

impl Serialize for () {
    fn read<R: Reader>(_reader: &mut R) -> Result<Self, ReadError> {
        Ok(())
    }

    fn write<W: Writer>(&self, _writer: &mut W) {}
}

impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let a = A::read(reader)?;
        let b = B::read(reader)?;
        Ok((a, b))
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        self.0.write(writer);
        self.1.write(writer);
    }
}

impl<A: Serialize, B: Serialize, C: Serialize> Serialize for (A, B, C) {
    fn read<R: Reader>(reader: &mut R) -> Result<Self, ReadError> {
        let a = A::read(reader)?;
        let b = B::read(reader)?;
        let c = C::read(reader)?;
        Ok((a, b, c))
    }

    fn write<W: Writer>(&self, writer: &mut W) {
        self.0.write(writer);
        self.1.write(writer);
        self.2.write(writer);
    }
}

#[allow(dead_code)]
pub struct Empty;

impl Game for Empty {
    type World = ();
    type Input = ();

    fn initial_world() -> Self::World { () }
    fn update_world(_: &Self::World) -> Self::World { () }
    fn update_player(_: &Self::World, _: PlayerId, _: &Self::Input) -> Self::World { () }
    fn add_player(_: &Self::World, _: PlayerId) -> Self::World { () }
    fn remove_player(_: &Self::World, _: PlayerId) -> Self::World { () }
    fn create_input(_: KeyboardState) -> Self::Input { () }
    fn render(_: &Self::World, _: PlayerId, _: u32, _: u32) {}
}