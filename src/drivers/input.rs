use std::{ops::Deref, sync::mpsc::Receiver};

use sdl2::{keyboard::Keycode, EventPump};
use thiserror::Error;
use wheat::{traits::Input, Key};

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
pub struct SdlInput {
    input_impl: SdlInputImpl,
    event_pump: EventPump,
    rx: Receiver<()>,
}

impl SdlInput {
    /// Creates a new `Input` with all key states set to `false`.
    pub fn new(sdl: &sdl2::Sdl, rx: Receiver<()>) -> Self {
        let event_pump = sdl.event_pump().unwrap();
        SdlInput {
            input_impl: SdlInputImpl::new(),
            event_pump,
            rx,
        }
    }

    pub fn update(&mut self) -> InputUpdate {
        if let Some(event) = self.event_pump.poll_event() {
            use sdl2::event::Event;
            if let Event::Quit { .. } = event {
                return InputUpdate::Quit;
            }
        }

        // Don't update input every frame; otherwise input is very janky
        while let Ok(()) = self.rx.try_recv() {
            let keys_pressed: Vec<_> = self
                .event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect();

            for i in 0..self.input_impl.keys.len() {
                self.input_impl.keys[i] = false;
            }

            for k in keys_pressed {
                if let Ok(chip8_key) = <Keycode as TryInto<Chip8Key>>::try_into(k) {
                    self.input_impl.keys[*chip8_key as usize] = true;
                }
            }
        }

        InputUpdate::Continue
    }

    pub fn input(&self) -> &SdlInputImpl {
        &self.input_impl
    }
}

#[derive(Debug)]
pub enum InputUpdate {
    Continue,
    Quit,
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Unsupported key")]
    UnsupportedKey,
}

struct Chip8Key(Key);

impl Deref for Chip8Key {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Keycode> for Chip8Key {
    type Error = InputError;

    fn try_from(value: Keycode) -> Result<Self, Self::Error> {
        match value {
            Keycode::Num1 => Ok(Chip8Key(Key::Num1)),
            Keycode::Num2 => Ok(Chip8Key(Key::Num2)),
            Keycode::Num3 => Ok(Chip8Key(Key::Num3)),
            Keycode::Num4 => Ok(Chip8Key(Key::C)),
            Keycode::Q => Ok(Chip8Key(Key::Num4)),
            Keycode::W => Ok(Chip8Key(Key::Num5)),
            Keycode::E => Ok(Chip8Key(Key::Num6)),
            Keycode::R => Ok(Chip8Key(Key::D)),
            Keycode::A => Ok(Chip8Key(Key::Num7)),
            Keycode::S => Ok(Chip8Key(Key::Num8)),
            Keycode::D => Ok(Chip8Key(Key::Num9)),
            Keycode::F => Ok(Chip8Key(Key::E)),
            Keycode::Z => Ok(Chip8Key(Key::A)),
            Keycode::X => Ok(Chip8Key(Key::Num0)),
            Keycode::C => Ok(Chip8Key(Key::B)),
            Keycode::V => Ok(Chip8Key(Key::F)),
            _ => Err(InputError::UnsupportedKey),
        }
    }
}

pub struct SdlInputImpl {
    pub(self) keys: Vec<bool>,
}

impl SdlInputImpl {
    fn new() -> Self {
        Self {
            keys: vec![false; NUM_KEYS],
        }
    }
}

impl Input for SdlInputImpl {
    fn is_pressed(&self, key: Key) -> bool {
        self.keys[key as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::{Chip8Key, SdlInputImpl};
    use sdl2::keyboard::Keycode;
    use wheat::{traits::Input, Key};

    macro_rules! update_test {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input_key, input_val) = $value;
                    let mut input = SdlInputImpl::new();
                    input.keys[*(<Keycode as TryInto<Chip8Key>>::try_into(input_key).unwrap()) as usize] = true;
                    assert_eq!(input.is_pressed(input_val.try_into().unwrap()), true);
                }
            )*
        }
    }

    update_test! {
        test_num1: (Keycode::Num1, Key::Num1),
        test_num2: (Keycode::Num2, Key::Num2),
        test_num3: (Keycode::Num3, Key::Num3),
        test_num4: (Keycode::Num4, Key::C),
        test_q: (Keycode::Q, Key::Num4),
        test_w: (Keycode::W, Key::Num5),
        test_e: (Keycode::E, Key::Num6),
        test_r: (Keycode::R, Key::D),
        test_a: (Keycode::A, Key::Num7),
        test_s: (Keycode::S, Key::Num8),
        test_d: (Keycode::D, Key::Num9),
        test_f: (Keycode::F, Key::E),
        test_z: (Keycode::Z, Key::A),
        test_x: (Keycode::X, Key::Num0),
        test_c: (Keycode::C, Key::B),
        test_v: (Keycode::V, Key::F),
    }
}
