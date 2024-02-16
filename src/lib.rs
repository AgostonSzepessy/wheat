use derive_builder::Builder;
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

#[derive(Error, Debug, PartialEq)]
pub enum Chip8Error {
    #[error("Internal error from unsupported key code: `{0}`")]
    InternalKeyError(u8),
    #[error("Rom could not be loaded fully into memory; stopping at `{0:#x}`")]
    RomTooBig(u16),
    #[error("Opcode `{0:#06x}` is not supported")]
    UnsupportedOpcode(u16),
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

/// Chip 8 has various quirks that differ from extension to extension.
/// This struct contains them, and can be adjusted depending on the game
/// being run.
///
/// A `Default` implementation is provided for the original Chip 8 platform.
#[derive(Debug, Builder)]
#[builder(default)]
pub struct Quirks {
    /// Should the `AND`, `OR`, and `XOR` instructions reset the `VF` register?
    pub reset_vf: bool,

    /// Should the `Fx55` and `Fx65` opcodes increment the index register? The
    /// original COSMAC VIP incremented the index register for these opcodes.
    /// Games from the 1970s and 1980s might rely on it being incremented.
    /// Modern games might rely on it not being incremented.
    pub increment_ir: bool,

    /// This applies to the shift instructions, `8XY6` and `8XYE`. Should register `VX` be
    /// set to the value of register `VY` before shifting?
    /// The original COSMAC VIP would set `VX` to `VY` and then perform the shift. Starting with
    /// CHIP-48 and SUPER-CHIP, `VX` was shifted in place, and `VY` was ignored completely.
    pub use_vy_in_shift: bool,
}

impl Quirks {
    pub fn new(reset_vf: bool, increment_ir: bool, shift: bool) -> Self {
        Self {
            reset_vf,
            increment_ir,
            use_vy_in_shift: shift,
        }
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Self {
            reset_vf: true,
            increment_ir: true,
            use_vy_in_shift: true,
        }
    }
}
