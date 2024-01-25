pub(self) use crate::traits::Input;
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
#[derive(Debug)]
pub struct SdlInput {
    keys: Vec<bool>,
}

impl SdlInput {
    /// Creates a new `Input` with all key states set to `false`.
    pub fn new() -> Self {
        SdlInput {
            keys: vec![false; NUM_KEYS],
        }
    }
}

impl Input for SdlInput {
    type Key = Keycode;

    /// Updates the state of the keys. `key` is the key to update, and `state`
    /// is the new state of the `key`.
    fn update(&mut self, key: &Keycode, state: bool) {
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
            _ => {}
        }
    }

    /// Returns the state of the specified key. The hex code that the key is
    /// mapped to is used to access its state.
    /// To check if `Num1` is pressed:
    ///
    /// ```
    /// use chip8::traits::Input;
    /// use chip8::input::SdlInput;
    ///
    /// let input = SdlInput::new();
    /// assert_eq!(input.is_pressed(&0x0), false);
    /// ```
    fn is_pressed(&self, key: &u8) -> bool {
        self.keys[*key as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::SdlInput;
    use crate::traits::Input;
    use sdl2::keyboard::Keycode;

    macro_rules! update_test {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input_key, input_val) = $value;
                    let mut input = SdlInput::new();
                    input.update(&input_key, true);
                    assert_eq!(input.is_pressed(&input_val), true);
                }
            )*
        }
    }

    update_test! {
        test_num1: (Keycode::Num1, 0x1),
        test_num2: (Keycode::Num2, 0x2),
        test_num3: (Keycode::Num3, 0x3),
        test_num4: (Keycode::Num4, 0xC),
        test_q: (Keycode::Q, 0x4),
        test_w: (Keycode::W, 0x5),
        test_e: (Keycode::E, 0x6),
        test_r: (Keycode::R, 0xD),
        test_a: (Keycode::A, 0x7),
        test_s: (Keycode::S, 0x8),
        test_d: (Keycode::D, 0x9),
        test_f: (Keycode::F, 0xE),
        test_z: (Keycode::Z, 0xA),
        test_x: (Keycode::X, 0x0),
        test_c: (Keycode::C, 0xB),
        test_v: (Keycode::V, 0xF),
    }
}
