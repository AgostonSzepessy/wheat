extern crate rand;
extern crate sdl2;

pub mod chip8;
pub mod display;
pub mod graphics;
pub mod input;

/// Screen is 64 pixels wide
const SCREEN_WIDTH: u16 = 64;
/// Screen is 32 pixels wide
const SCREEN_HEIGHT: u16 = 32;
const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
/// All sprites are 8 pixels wide
const SPRITE_WIDTH: u16 = 8;
