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
