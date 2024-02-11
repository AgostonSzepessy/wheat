use crate::traits::GraphicsBuffer;
use crate::{SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH, SPRITE_WIDTH};

const PIXEL_OFF: u8 = 0;
const PIXEL_ON: u8 = 1;

/// Graphics processor for Chip8. The emulator has a screen that is `64`x`32` pixels.
///
/// All sprites drawn on it are `8` pixels wide, with each pixel being `1` bit, so there are `8` pixels
/// in `1` byte. The position and height of each sprite is determined by the opcode `0xDxyn`, where
/// `D` is the symbol for draw, `x` is the `x` position, `y` is the `y` position, and `n` is
/// the height of the sprite.
///
/// Sprites are `XOR`ed onto the screen, and if a pixel flips from `1` to `0`, it is signalled in
/// the `VF` register.
#[derive(Debug)]
pub struct Graphics {
    /// Screen on which sprites are drawn
    screen: Vec<Vec<u8>>,
}

impl Graphics {
    /// Creates a new Graphics, with a screen of `64`x`32` pixels, and all pixels on the screen
    /// initialized to 0.
    pub fn new() -> Self {
        Graphics {
            screen: vec![vec![0; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
        }
    }

    fn dump(&self) {
        for i in 0..self.screen.len() {
            for j in 0..self.screen[0].len() {
                print!("{} ", self.screen[i][j]);
            }
            println!("");
        }
    }
}

impl GraphicsBuffer for Graphics {
    /// Clears the entire screen with 0s; wipes everything from the screen.
    fn clear(&mut self) {
        for i in 0..self.screen.len() {
            for j in 0..self.screen[0].len() {
                self.screen[i][j] = 0;
            }
        }
    }

    /// Draws a sprite on the screen, and returns `true` if a pixel on the screen was flipped from
    /// 1 to 0.
    /// `opcode`: Determines position and height of sprite, with position being top left
    /// corner of the sprite.
    /// `ir`: The index register, which contains the area of memory to
    /// start reading the sprite from.
    /// `memory`: The memory from which to read the sprite.
    fn draw(&mut self, opcode: &u16, ir: &u16, memory: &Vec<u8>) -> bool {
        // x and y position, and height of the sprite, with the origin at the
        // top left corner
        let x = ((*opcode & 0x0F00) >> 8) % SCREEN_WIDTH;
        let y = ((*opcode & 0x00F0) >> 4) % SCREEN_HEIGHT;
        let num_rows = *opcode & 0x000F;

        // Assume no collisions happen
        let mut pixel_flipped = false;

        // Width of each pixel is 8 bits, and height is determined by the last nibble in opcode
        for row in 0..num_rows {
            let sprite = memory[(*ir + row) as usize];
            println!("{:#010b}", sprite);
            for bit in 0..SPRITE_WIDTH {
                // Keep only the smallest bit, because that's what we care about
                let pixel = (sprite >> (7 - bit)) & 0x1;

                // Allow wrap-around by modulusing the result
                let pos_y = (y + row) as usize;
                let pos_x = (x + bit) as usize;

                if pixel == PIXEL_ON && self.screen[pos_y][pos_x] == PIXEL_ON {
                    self.screen[pos_y][pos_x] ^= pixel;
                    pixel_flipped = true;
                } else {
                    self.screen[pos_y][pos_x] ^= pixel;
                }
            }
        }

        self.dump();

        pixel_flipped
    }

    fn buffer(&self) -> &Vec<Vec<u8>> {
        &self.screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_clear() {
    //     let mut graphics = Graphics::new();
    //     graphics.clear();

    //     for i in 0..graphics.screen.len() {
    //         assert_eq!(graphics.screen[i], 0);
    //     }
    // }
}
