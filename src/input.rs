use sdl2::keyboard::Keycode;

const NUM_KEYS: usize = 16;

/// Keeps track of the state of the keys. Chip8 uses 16 keys; this implementation
/// defines the following:
///
/// | Keys   | Keys   | Keys   | Keys   |
/// |--------|--------|--------|--------|
/// | 1 (0x1) | 2 (0x2) | 3 (0x3) | 4 (0xC) |
/// | Q (0x4) | W (0x5) | E (0x6) | R (0xD) |
/// | A (0x7) | S (0x8) | D (0x9) | F (0xE) |
/// | Z (0xA) | X (0x0) | C (0xB) | V (0xF) |
///
/// based off of this diagram: <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard>
pub struct Input {
    keys: Vec<bool>,
}

impl Input {
    /// Creates a new `Input` with all key states set to `false`.
    pub fn new() -> Self {
        Input {
            keys: vec![false; NUM_KEYS],
        }
    }

    /// Updates the state of the keys. `key` is the key to update, and `state`
    /// is the new state of the `key`.
    pub fn update(&mut self, key: &Keycode, state: bool) {
        match *key {
            Keycode::Num1 => self.keys[0x1] = state,
            Keycode::Num2 => self.keys[0x2] = state,
            Keycode::Num3 => self.keys[0x3] = state,
            Keycode::Num4 => self.keys[0xC] = state,
            Keycode::Q => self.keys[0x4] = state,
            Keycode::W => self.keys[0x5] = state,
            Keycode::E => self.keys[0x6] = state,
            Keycode::R => self.keys[0xD] = state,
            Keycode::A => self.keys[0x7] = state,
            Keycode::S => self.keys[0x8] = state,
            Keycode::D => self.keys[0x9] = state,
            Keycode::F => self.keys[0xE] = state,
            Keycode::Z => self.keys[0xA] = state,
            Keycode::X => self.keys[0x0] = state,
            Keycode::C => self.keys[0xB] = state,
            Keycode::V => self.keys[0xF] = state,
            _ => { }
        }
    }

    /// Returns the state of the specified key. The hex code that the key is
    /// mapped to is used to access its state.
    /// To check if `Num1` is pressed:
    ///
    /// ```
    /// let input = Input::new();
    /// assert_eq!(input.is_pressed(0x0), false);
    /// ```
    pub fn is_pressed(&self, key: &u8) -> bool {
        self.keys[*key as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::Input;
    use sdl2::keyboard::Keycode;

    #[test]
    fn test_num1() {
        let mut input = Input::new();
        input.update(&Keycode::Num1, true);
        assert_eq!(input.is_pressed(&0x1), true);
    }

    #[test]
    fn test_num2() {
        let mut input = Input::new();
        input.update(&Keycode::Num2, true);
        assert_eq!(input.is_pressed(&0x2), true);
    }

    #[test]
    fn test_num3() {
        let mut input = Input::new();
        input.update(&Keycode::Num3, true);
        assert_eq!(input.is_pressed(&0x3), true);
    }

    #[test]
    fn test_num4() {
        let mut input = Input::new();
        input.update(&Keycode::Num4, true);
        assert_eq!(input.is_pressed(&0xC), true);
    }
}
