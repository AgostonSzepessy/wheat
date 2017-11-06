/// The graphics processing of the Chip8. The emulator has a screen that is `64`x`32` pixels.
///
/// All sprites drawn on it are 8 pixels wide, with each pixel being 1 bit, hence 8 pixels in 1
/// byte. The position and height of each sprite is determined by the opcode `0xDxyn`, where 
/// `D` is the symbol for draw, `x` is the `x` position, `y` is the `y` position, and `n` is 
/// the height of the sprite.
///
/// Sprites are XORed onto the screen, and if a pixel flips from 1 to 0, it is signalled in
/// the `VF` register.
#[derive(Debug)]
pub struct Graphics {
    /// Screen on which sprites are drawn
    screen: Vec<u8>,
}

/// Screen is 64 pixels wide
const SCREEN_WIDTH: u16 = 64;
/// Screen is 32 pixels wide
const SCREEN_HEIGHT: u16 = 32;
const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
/// All sprites are 8 pixels wide
const SPRITE_WIDTH: u16 = 8;

impl Graphics {
    /// Creates a new Graphics, with a screen of `64`x`32` pixels, and all pixels on the screen
    /// initialized to 0.
    pub fn new() -> Self {
        Graphics {
            screen: vec![0; SCREEN_SIZE as usize],
        }
    }

    /// Clears the entire screen with 0s; wipes everything from the screen.
    pub fn clear(&mut self) {
        for i in 0..self.screen.len() {
            self.screen[i] = 0;
        }
    }

    /// Draws a sprite on the screen, and returns `true` if a pixel on the screen was flipped from
    /// 1 to 0. `opcode`: Determines position and height of sprite, with position being top left
    ///   corner of the sprite. `ir`: The index register, which contains the area of memory to
    ///   start reading the sprite from. `memory`: The memory from which to read the sprite.
    pub fn draw(&mut self, opcode: &u8, ir: &u8, memory: &Vec<u8>) -> bool {
        // x and y position, and height of the sprite, with the origin at the 
        // top left corner
        let x: u16 = ((*opcode as u16) & 0x0F00) >> 8;
        let y: u16 = ((*opcode as u16) & 0x00F0) >> 4;
        let num_rows: u16 = (*opcode as u16) & 0x000F;

        // Assume no collisions happen
        let mut pixel_flipped = false;

        // Width of each pixel is 8 bits, and height is determined by the last nibble in opcode
        for row in 0..num_rows {
            for col in 0..SPRITE_WIDTH {
                // Check if the bit is set. Sprites are layed out in memory starting
                // with the top left corner. 0x80 = 128, so we begin drawing from the
                // top left corner
                if memory[(*ir as u16 + row) as usize] & (0x80 >> col) != 0 {
                    // If a bit changes from 1 to 0, we need to signal it in the "carry" bit
                    if self.screen[((((row + y) * SCREEN_WIDTH) % SCREEN_HEIGHT) + ((col + x) % SCREEN_WIDTH)) as usize] == 1 {
                        pixel_flipped = true;
                    }
                    // This math maps a multidimensional array index to a single dimensional array,
                    // and the modulus takes care of wrapping around the screen if the index goes 
                    // past it
                    self.screen[((((row + y) * SCREEN_WIDTH) % SCREEN_HEIGHT) + ((col + x) % SCREEN_WIDTH)) as usize] ^= 1;
                }
            }
        }

        pixel_flipped
    }
}
