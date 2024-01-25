pub mod chip8;
pub mod graphics;
pub mod input;
pub mod traits;

/// Screen is 64 pixels wide
pub const SCREEN_WIDTH: u16 = 64;
/// Screen is 32 pixels wide
pub const SCREEN_HEIGHT: u16 = 32;
pub const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
/// All sprites are 8 pixels wide
pub const SPRITE_WIDTH: u16 = 8;
