pub trait Graphics {
    /// Clears the entire screen with 0s; wipes everything from the screen.
    fn clear(&mut self);

    /// Draws a sprite on the screen, and returns `true` if a pixel on the screen was flipped from
    /// 1 to 0.
    /// `opcode`: Determines position and height of sprite, with position being top left
    /// corner of the sprite.
    /// `ir`: The index register, which contains the area of memory to
    /// start reading the sprite from.
    /// `memory`: The memory from which to read the sprite.
    fn draw(&mut self, opcode: &u16, ir: &u16, memory: &Vec<u8>) -> bool;

    fn screen(&self) -> &Vec<u8>;
}

pub trait Display {
    /// Draws the specified `buffer`. The buffer is expected to be
    /// made up of `1`s and `0`s. `1`s are drawn as white and `0`s
    /// are drawn as black.
    fn draw(&mut self, buffer: &Vec<u8>);
}

/// Keeps track of the state of the keys. Chip8 uses 16 keys; this implementation
/// relies on  the following mapping:
///
/// | Keys   | Keys   | Keys   | Keys   |
/// |--------|--------|--------|--------|
/// | 1 (0x1) | 2 (0x2) | 3 (0x3) | 4 (0xC) |
/// | Q (0x4) | W (0x5) | E (0x6) | R (0xD) |
/// | A (0x7) | S (0x8) | D (0x9) | F (0xE) |
/// | Z (0xA) | X (0x0) | C (0xB) | V (0xF) |
///
/// based off of this diagram: <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard>
pub trait Input {
    type Key;

    /// Updates the state of the keys. `key` is the key to update, and `state`
    /// is the new state of the `key`.
    fn update(&mut self, key: &Self::Key, state: bool);

    /// Returns the state of the specified key. The hex code that the key is
    /// mapped to is used to access its state.
    fn is_pressed(&self, key: &u8) -> bool;
}
