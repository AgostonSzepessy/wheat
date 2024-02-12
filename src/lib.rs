use thiserror::Error;

pub mod chip8;
pub mod graphics;
pub mod input;
pub mod timer;
pub mod traits;

/// Screen is 64 pixels wide
pub const SCREEN_WIDTH: u16 = 64;
/// Screen is 32 pixels wide
pub const SCREEN_HEIGHT: u16 = 32;
pub const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
/// All sprites are 8 pixels wide
pub const SPRITE_WIDTH: u8 = 8;

/// The keymap that this implementation uses internally. Based off
/// of: <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard>
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Key {
    Num0 = 0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
}

#[derive(Error, Debug)]
pub enum Chip8Error {
    #[error("Internal error from unsupported key code: `{0}`")]
    InternalKeyError(u8),
    #[error("Rom could not be loaded fully into memory; stopping at `{0:#x}`")]
    RomTooBig(u16),
}

impl TryFrom<u8> for Key {
    type Error = Chip8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Key::Num0),
            1 => Ok(Key::Num1),
            2 => Ok(Key::Num2),
            3 => Ok(Key::Num3),
            4 => Ok(Key::Num4),
            5 => Ok(Key::Num5),
            6 => Ok(Key::Num6),
            7 => Ok(Key::Num7),
            8 => Ok(Key::Num8),
            9 => Ok(Key::Num9),
            0xA => Ok(Key::A),
            0xB => Ok(Key::B),
            0xC => Ok(Key::C),
            0xD => Ok(Key::D),
            0xE => Ok(Key::E),
            0xF => Ok(Key::F),
            e => Err(Chip8Error::InternalKeyError(e)),
        }
    }
}
