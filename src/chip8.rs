use core::num;
use std::sync::mpsc::Receiver;

use rand::Rng;

use crate::timer::TimerOperation;
use crate::traits::{GraphicsBuffer, Input, Rom};
use crate::{Chip8Error, Key};

#[derive(Debug)]
pub struct Chip8<G> {
    /// Current opcode
    opcode: u16,
    /// The system has 4096 bytes of memory.
    memory: Vec<u8>,
    /// The index register (I)
    ir: u16,
    /// The program counter
    pc: u16,
    delay_timer: u8,
    registers: Vec<u8>,
    /// When this timer reaches 0, the system's buzzer sounds
    sound_timer: u8,
    /// Function call stack. When a jump is performed, the current location
    /// is pushed on the stack so it can be retrieved later.
    stack: Vec<u16>,
    /// The stack pointer
    sp: u8,
    /// Screen that sprites get drawn on. 64x32 pixels
    graphics: G,
    timer_rx: Receiver<TimerOperation>,
    draw_on_screen: bool,
    wait_for_keypress_register: u8,
    wait_for_key_state: WaitForKeyState,
}

// The default address at which the application is loaded at
const APP_LOCATION: u16 = 0x200;

// Total memory available to Chip8
const MEMORY_SIZE: usize = 4096;

// Total size of the stock
const STACK_SIZE: usize = 16;

// Number of registers available
const NUM_REGISTERS: usize = 16;

// Register size in bytes.
const REG_SIZE: u16 = 1;

const OPCODE_SIZE: u16 = 2;

const FLAG_REGISTER: usize = 0xF;

/// Used for keycode `0xFX0A` (wait for keypress). This opcode
/// requires halting the whole emulator until a key is pressed
/// and released. This is part of a state machine that achieves that.
#[derive(PartialEq, Debug)]
enum WaitForKeyState {
    None,
    WaitForNoKeyPressed,
    CheckForKeyPressed,
    WaitForKeyRelease,
}

#[derive(Debug, PartialEq)]
enum ProgramCounter {
    None,
    Next,
    Skip,
    Set(u16),
    Pause,
}

// Chip8 provides hexadecimal digit sprites stored in memory from 0x000 to
// 0x1FF.
const HEX_DIGITS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // Number: 0
    0x20, 0x60, 0x20, 0x20, 0x70, // Number: 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // Number: 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // Number: 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // Number: 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // Number: 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // Number: 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // Number: 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // Number: 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // Number: 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // Letter: A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // Letter: B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // Letter: C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // Letter: D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // Letter: E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // Letter: F
];

pub struct Chip8OutputState<'a> {
    pub sound_on: bool,
    pub draw_on_screen: bool,
    pub graphics: &'a dyn GraphicsBuffer,
}

impl<'a> Chip8OutputState<'a> {
    pub fn new(sound_on: bool, draw_on_screen: bool, graphics_buffer: &'a dyn GraphicsBuffer) -> Self {
        Self {
            sound_on,
            draw_on_screen,
            graphics: graphics_buffer,
        }
    }
}

type OpcodeResult = Result<ProgramCounter, Chip8Error>;

// Throughout the code, Vx refers to the general purpose registers. There are
// 15 general purpose registers from V0 to VE. The 16th register is used to
// represent the carry flag.

impl<G> Chip8<G>
where
    G: GraphicsBuffer,
{
    pub fn new(graphics: G, timer_rx: Receiver<TimerOperation>) -> Self {
        let mut memory = vec![0; MEMORY_SIZE];

        for i in 0..HEX_DIGITS.len() {
            memory[i] = HEX_DIGITS[i];
        }

        Chip8 {
            opcode: 0,
            memory,
            ir: 0,
            pc: APP_LOCATION,
            graphics,
            delay_timer: 0,
            registers: vec![0; NUM_REGISTERS],
            sound_timer: 0,
            stack: vec![0; STACK_SIZE],
            sp: 0,
            timer_rx,
            draw_on_screen: false,
            wait_for_keypress_register: 0,
            wait_for_key_state: WaitForKeyState::None,
        }
    }

    pub fn load_rom(&mut self, rom: &impl Rom) -> Result<(), Chip8Error> {
        for (i, rom_data) in rom.data().iter().enumerate() {
            let addr = APP_LOCATION as usize + i;
            if i < MEMORY_SIZE {
                self.memory[addr] = *rom_data;
            } else {
                return Err(Chip8Error::RomTooBig(addr as u16));
            }
        }

        Ok(())
    }

    pub fn emulate_cycle(&mut self, input: &impl Input) -> Result<Chip8OutputState, Chip8Error> {
        self.draw_on_screen = false;

        let input_result = self.check_and_process_0xfx0a(input)?;
        let mut stack_operation = ProgramCounter::None;

        if input_result != ProgramCounter::Pause {
            stack_operation = self.emulate_instruction(input)?;
        }

        match stack_operation {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Skip => self.pc += OPCODE_SIZE * 2,
            ProgramCounter::Set(addr) => self.pc = addr,
            ProgramCounter::None | ProgramCounter::Pause => (),
        }

        // If there's a timer message, update the timers
        while let Ok(timer_operation) = self.timer_rx.try_recv() {
            match timer_operation {
                TimerOperation::Decrement(val) => {
                    self.sound_timer = self.sound_timer.saturating_sub(val);
                    self.delay_timer = self.delay_timer.saturating_sub(val);
                }
            }
        }

        let sound_on = self.sound_timer > 0;
        Ok(Chip8OutputState::new(
            sound_on,
            self.draw_on_screen,
            &self.graphics,
        ))
    }

    fn emulate_instruction(&mut self, input: &impl Input) -> OpcodeResult {
        self.opcode =
            ((self.memory[self.pc as usize] as u16) << 8) | self.memory[self.pc as usize + 1] as u16;

        if self.opcode != 0xf00a {
            println!("opcode is {:#x}", self.opcode);
        }

        match self.opcode & 0xF000 {
            // Opcode starts with 0x0
            0x0000 => self.opcode_0x0yyy(),

            // Opcode starts with 0x1
            0x1000 => self.opcode_0x1yyy(),

            // Opcode starts with 0x2
            0x2000 => self.opcode_0x2yyy(),

            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx == kk
            0x3000 => self.opcode_0x3yyy(),

            // Opcodes that start with 0x4
            0x4000 => self.opcode_0x4yyy(),

            // Opcodes that start with 0x5
            0x5000 => self.opcode_0x5yyy(),

            // Opcodes that start with 0x6
            0x6000 => self.opcode_0x6yyy(),

            // Opcodes that start with 0x7
            0x7000 => self.opcode_0x7yyy(),

            // Opcodes that start with 0x8
            0x8000 => self.opcode_0x8yyy(),

            // Opcodes that start with 0x9
            0x9000 => self.opcode_0x9yyy(),

            // Opcodes that start with 0xA
            0xA000 => self.opcode_0xayyy(),

            // Opcodes that start with 0xB
            0xB000 => self.opcode_0xbyyy(),

            // Cxkk - RND, byte
            // Set Vx = random byte AND kk
            // Interpreter generates a random number between 0 and 255, which
            // is then ANDed with kk and the result is stored in Vx.
            0xC000 => self.opcode_0xcyyy(),

            0xD000 => self.opcode_0xdyyy(),

            0xE000 => self.opcode_0xeyyy(input),

            0xF000 => self.opcode_0xfyyy(),

            _ => self.unknown_opcode(),
        }
    }

    // Utility function to return the number of registers x and y.
    fn get_regs_x_y(&self) -> (usize, usize) {
        return (
            ((self.opcode & 0x0F00) >> 8) as usize,
            ((self.opcode & 0x00F0) >> 4) as usize,
        );
    }

    fn unknown_opcode(&mut self) -> OpcodeResult {
        println!("unknown opcode: {:X}", self.opcode);
        Err(Chip8Error::UnsupportedOpcode(self.opcode))
    }

    /// Takes care of opcodes that start with 0x0.
    fn opcode_0x0yyy(&mut self) -> OpcodeResult {
        match self.opcode & 0x00FF {
            // Clear the screen
            0x00E0 => {
                self.graphics.clear();
                self.draw_on_screen = true;
                Ok(ProgramCounter::Next)
            }
            // Return from subroutine
            0x00EE => {
                // Restore program counter to previous location on stack
                // before subroutine was called
                self.sp -= 1;
                Ok(ProgramCounter::Set(self.stack[self.sp as usize]))
            }

            // No other opcodes start with 0x0
            _ => self.unknown_opcode(),
        }
    }

    /// Takes care of opcodes that start with 0x1.
    fn opcode_0x1yyy(&mut self) -> OpcodeResult {
        // Only 1 opcode that starts with 0x1: 0x1nnn
        // 0x1nnn - Jump to location nnn
        let addr = self.opcode & 0x0FFF;
        Ok(ProgramCounter::Set(addr))
    }

    /// Takes care of opcodes that start with 0x2.
    fn opcode_0x2yyy(&mut self) -> OpcodeResult {
        // 0x2adr - Call subroutine at adr
        // Put instruction after program counter on stack and then jump to subroutine
        // location. This prevents the VM from entering into an endless loop.
        self.stack[self.sp as usize] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        let addr = self.opcode & 0x0FFF;
        Ok(ProgramCounter::Set(addr))
    }

    /// Takes care of opcodes that start with 0x3.
    fn opcode_0x3yyy(&mut self) -> OpcodeResult {
        // 3xkk - SE Vx, byte
        // Skip next instruction if Vx == kk

        // Get register value and constant
        let (x, _) = self.get_regs_x_y();
        let register_val = self.registers[x];
        let comp_val = (self.opcode & 0x00FF) as u8;

        // If equal, skip next instruction (increment program
        // counter by 2)
        if register_val == comp_val {
            return Ok(ProgramCounter::Skip);
        }

        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0x4.
    fn opcode_0x4yyy(&mut self) -> OpcodeResult {
        // 4xkk - SNE Vx, byte
        // Skip next instruction if Vx != kk

        // Get register value and constant
        let (x, _) = self.get_regs_x_y();
        let register_val = self.registers[x];
        let comp_val = (self.opcode & 0x00FF) as u8;

        // If not equal, skip next instruction (increment program
        // counter by 2)
        if register_val != comp_val {
            return Ok(ProgramCounter::Skip);
        }

        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0x5.
    fn opcode_0x5yyy(&mut self) -> OpcodeResult {
        // 5xy0 - SE Vx, Vy
        // Skip next instruction if Vx == Vy
        let (x, y) = self.get_regs_x_y();
        let vx_val = self.registers[x];
        let vy_val = self.registers[y];

        // If values are equal, skip next instruction (increment
        // program counter by 2)
        if vx_val == vy_val {
            return Ok(ProgramCounter::Skip);
        }

        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0x6.
    fn opcode_0x6yyy(&mut self) -> OpcodeResult {
        // 6xkk - LD Vx, byte
        // Set Vx = kk
        let val = (self.opcode & 0x00FF) as u8;
        let (x, _) = self.get_regs_x_y();

        // Set register to value
        self.registers[x] = val;
        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0x7.
    fn opcode_0x7yyy(&mut self) -> OpcodeResult {
        // 7xkk - ADD Vx, byte
        // Set Vx = Vx + kk
        // Get value and register
        let val = (self.opcode & 0x00FF) as u8;
        let x = ((self.opcode & 0x0F00) >> 8) as usize;

        self.registers[x] = self.registers[x].wrapping_add(val);
        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0x8.
    fn opcode_0x8yyy(&mut self) -> OpcodeResult {
        // Last nibble identifies what the opcode does
        match self.opcode & 0x000F {
            // 8xy0 - LD Vx, Vy
            // Set Vx = Vy
            0x0000 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] = self.registers[y];
                Ok(ProgramCounter::Next)
            }

            // 8xy1 - OR Vx, Vy
            // Perform bitwise OR on Vx and Vy and store result in Vx.
            0x0001 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] |= self.registers[y];
                Ok(ProgramCounter::Next)
            }

            // 8xy2 - AND Vx, Vy
            // Perform bitwise AND on Vx and Vy and store result in Vx.
            0x0002 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] &= self.registers[y];
                Ok(ProgramCounter::Next)
            }

            // 8xy3 - XOR Vx, Vy
            // Performs bitwise XOR on Vx and Vy and stores result in Vx.
            0x0003 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] ^= self.registers[y];
                Ok(ProgramCounter::Next)
            }

            // 8xy4 - ADD Vx, Vy
            // Vx = Vx + Vy, set VF = carry
            // If the result of Vx and Vy is greater than 8 bits (255)
            // VF is set to 1, otherwise it's set to 0
            0x0004 => {
                let (x, y) = self.get_regs_x_y();
                let (val, overflow) = self.registers[x].overflowing_add(self.registers[y]);

                let flag = if overflow { 1 } else { 0 };

                self.registers[x] = val;
                self.registers[FLAG_REGISTER] = flag;

                Ok(ProgramCounter::Next)
            }

            // 8xy5 - SUB Vx, Vy
            // Vx= Vx - Vy, set VF = NOT borrow
            // If Vx > Vy, then VF is set to 1, otherwise 0
            0x0005 => {
                let (x, y) = self.get_regs_x_y();

                let flag = if self.registers[x] >= self.registers[y] {
                    1
                } else {
                    0
                };

                let (val, _) = self.registers[x].overflowing_sub(self.registers[y]);

                self.registers[x] = val;
                self.registers[FLAG_REGISTER] = flag;

                Ok(ProgramCounter::Next)
            }

            // 8xy6 - SHR Vx {, Vy}
            // Set Vx = Vx SHR 1
            // If least significant bit of Vx is 1, then VF is set to 1,
            // otherwise 0. Then Vx is divided by 2
            0x0006 => {
                let (x, _) = self.get_regs_x_y();

                let flag = self.registers[x] & 0x1;
                self.registers[x] >>= 1;

                self.registers[FLAG_REGISTER] = flag;
                Ok(ProgramCounter::Next)
            }

            // 8xy7 - SUBN Vx, Vy
            // Set Vx = Vy - Vx, set VF = NOT borrow
            // If Vy > Vx, then VF = 1, otherwise VF = 0.
            0x0007 => {
                let (x, y) = self.get_regs_x_y();

                let flag = if self.registers[y] >= self.registers[x] {
                    1
                } else {
                    0
                };

                let (val, _) = self.registers[y].overflowing_sub(self.registers[x]);

                self.registers[x] = val;
                self.registers[FLAG_REGISTER] = flag;

                Ok(ProgramCounter::Next)
            }

            // 8xyE - SHL Vx {, Vy}
            // Set Vx = Vx SHL 1
            // If most significant bit of Vx is 1, set VF to 1, otherwise 0.
            0x000E => {
                let (x, _) = self.get_regs_x_y();

                let flag = (self.registers[x] >> 7) & 0x1;

                self.registers[x] <<= 1;
                self.registers[FLAG_REGISTER] = flag;

                Ok(ProgramCounter::Next)
            }

            // No other opcodes start with 0x8
            _ => self.unknown_opcode(),
        }
    }

    /// Takes care of opcodes that start with 0x9
    fn opcode_0x9yyy(&mut self) -> OpcodeResult {
        // 9xy0 - SNE Vx, Vy
        // Skip next instruction if Vx != Vy
        let (x, y) = self.get_regs_x_y();

        if self.registers[x] != self.registers[y] {
            return Ok(ProgramCounter::Skip);
        }

        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0xA
    fn opcode_0xayyy(&mut self) -> OpcodeResult {
        // Annn - LD I, addr
        // Set I = nnn
        // Get address and set index register
        let val = self.opcode & 0x0FFF;
        self.ir = val;
        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0xB
    fn opcode_0xbyyy(&mut self) -> OpcodeResult {
        // Bnnn - JP V0, nnn
        // Jump to location nnn + V0 (set pc = nnn + V0)
        let val = self.opcode & 0x0FFF;
        Ok(ProgramCounter::Set(val + self.registers[0x0] as u16))
    }

    /// Takes care of opcodes that start with 0xC
    fn opcode_0xcyyy(&mut self) -> OpcodeResult {
        // Cxkk - RND, byte
        // Set Vx = random byte AND kk
        // Interpreter generates a random number between 0 and 255, which
        // is then ANDed with kk and the result is stored in Vx.
        let kk: u8 = (self.opcode & 0x00FF) as u8;
        let (x, _) = self.get_regs_x_y();

        let rand_val = rand::thread_rng().gen_range(0..256) as u8;

        self.registers[x] = rand_val & kk;
        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that start with 0xD
    fn opcode_0xdyyy(&mut self) -> OpcodeResult {
        // Dxyn - DRW Vx, Vy, nibble
        // Display n-byte sprite starting at memory location I at (Vx, Vy),
        // set VF = collision
        let (x_reg, y_reg) = self.get_regs_x_y();
        let num_rows = (self.opcode & 0x000F) as u8;

        let x = self.registers[x_reg];
        let y = self.registers[y_reg];

        let flipped = self.graphics.draw(x, y, num_rows, self.ir, &self.memory);
        self.draw_on_screen = true;

        if flipped {
            self.registers[FLAG_REGISTER] = 1;
        } else {
            self.registers[FLAG_REGISTER] = 0;
        }

        Ok(ProgramCounter::Next)
    }

    /// Takes care of opcodes that are related to input such as checking whether
    /// a key is pressed or not pressed, and waiting until a key is pressed.
    fn opcode_0xeyyy(&mut self, input: &impl Input) -> OpcodeResult {
        match self.opcode & 0xFF {
            // Ex9E - SKP Vx
            // Skips the next instruction if the key with the value of Vx is
            // pressed. If the key corresponding to the value of Vx is currently
            // in the down position, PC is increased by 2.
            0x9E => {
                let (x, _) = self.get_regs_x_y();

                if input.is_pressed((self.registers[x]).try_into().unwrap()) {
                    return Ok(ProgramCounter::Skip);
                }

                Ok(ProgramCounter::Next)
            }

            // ExA1 - SKNP Vx
            // Skip next instruction if key with value Vx is not pressed. If the
            // key with value Vx is not pressed, the program counter is incremented
            // by 2.
            0xA1 => {
                let (x, _) = self.get_regs_x_y();

                if !input.is_pressed((self.registers[x]).try_into().unwrap()) {
                    return Ok(ProgramCounter::Skip);
                }

                Ok(ProgramCounter::Next)
            }

            _ => self.unknown_opcode(),
        }
    }

    fn opcode_0xfyyy(&mut self) -> OpcodeResult {
        match self.opcode & 0xFF {
            // Fx07 - LD Vx, DT
            // Set Vx = delay timer value.
            // The value of DT is placed into Vx.
            0x07 => {
                let (x, _) = self.get_regs_x_y();
                self.registers[x] = self.delay_timer;
                println!("self.registers[x] = delay timer");
                println!("self.registers[{:#x}] = {}", x, self.delay_timer);
                Ok(ProgramCounter::Next)
            }

            // Fx0A - LD Vx, K
            // Wait for a key press, store the value of the key in Vx.
            // All execution stops until a key is pressed, then the value
            // of that key is stored in Vx.
            0x0A => {
                let (x, _) = self.get_regs_x_y();

                if self.wait_for_key_state == WaitForKeyState::None {
                    self.wait_for_keypress_register = x as u8;
                    self.wait_for_key_state = WaitForKeyState::WaitForNoKeyPressed;
                }

                Ok(ProgramCounter::Pause)
            }

            // Fx15 - LD DT, Vx
            // Set delay timer = Vx
            // DT is set equal to the value of Vx.
            0x15 => {
                let (x, _) = self.get_regs_x_y();
                self.delay_timer = self.registers[x];
                println!("delay timer = self.registers[x]");
                println!("delay timer = {} (register[{:#}]", self.registers[x], x);
                Ok(ProgramCounter::Next)
            }

            // Fx18 - LD ST, Vx
            // Set sound timer = Vx
            // ST is set equal to the value of Vx.
            0x18 => {
                let (x, _) = self.get_regs_x_y();
                self.sound_timer = self.registers[x];
                Ok(ProgramCounter::Next)
            }

            // Fx1E - ADD I, Vx
            // Set I = I + Vx
            // The values of I and Vx are added, and the results are stored in I.
            0x1E => {
                let (x, _) = self.get_regs_x_y();
                self.ir += self.registers[x] as u16;
                Ok(ProgramCounter::Next)
            }

            // Fx29 - LD F, Vx
            // Set I = location of sprite for digit Vx.
            // The value of I is set to the location for the hexadecimal sprite
            // corresponding to the value of Vx.
            0x29 => {
                let (x, _) = self.get_regs_x_y();
                // Each hex sprite takes up 5 bytes, and they start at address
                // 0x0, so multiplying the value in Vx by 5 will get us the
                // address of the sprite
                self.ir = self.registers[x] as u16 * 5;
                Ok(ProgramCounter::Next)
            }

            // Fx33 - LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1, and I+2.
            // The interpreter takes the decimal value of Vx, and places the hundreds digit
            // in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
            0x33 => {
                let (x, _) = self.get_regs_x_y();
                let val = self.registers[x];

                let hundreds = (val / 100) as u8;
                let tens = ((val / 10) % 10) as u8;
                let ones = (val % 10) as u8;

                self.memory[self.ir as usize] = hundreds;
                self.memory[self.ir as usize + 1] = tens;
                self.memory[self.ir as usize + 2] = ones;

                Ok(ProgramCounter::Next)
            }

            // Fx55 - LD [I], Vx
            // Store registers V0 through Vx in memory starting at location I.
            // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
            0x55 => {
                let (x, _) = self.get_regs_x_y();
                let mut addr = self.ir;

                for i in 0..=x {
                    self.memory[addr as usize] = self.registers[i];
                    addr += REG_SIZE;
                }

                Ok(ProgramCounter::Next)
            }

            // Fx65 - LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I.
            // The interpreter reads values from memory starting at location I into registers V0 through Vx.
            0x65 => {
                let (x, _) = self.get_regs_x_y();
                let mut addr = self.ir;

                for i in 0..=x {
                    self.registers[i] = self.memory[addr as usize];
                    addr += REG_SIZE;
                }

                Ok(ProgramCounter::Next)
            }

            _ => self.unknown_opcode(),
        }
    }

    // 0xFX0A requires special handling. It has to wait for the key
    // to be released before registering the key pressed. It also
    // needs to halt the whole emulator, except for timers.
    // Timers need to continue to decrement.
    fn check_and_process_0xfx0a(&mut self, input: &impl Input) -> OpcodeResult {
        if self.wait_for_key_state != WaitForKeyState::None {
            match self.wait_for_key_state {
                WaitForKeyState::WaitForNoKeyPressed => {
                    let mut key_pressed = false;
                    for i in 0..=Key::F as u8 {
                        if input.is_pressed(i.try_into().unwrap()) {
                            key_pressed = true;
                        }
                    }
                    if !key_pressed {
                        self.wait_for_key_state = WaitForKeyState::CheckForKeyPressed;
                    }
                    Ok(ProgramCounter::Pause)
                }
                WaitForKeyState::CheckForKeyPressed => {
                    for i in 0..=Key::F as u8 {
                        if input.is_pressed(i.try_into().unwrap()) {
                            self.registers[self.wait_for_keypress_register as usize] = i;
                            self.wait_for_key_state = WaitForKeyState::WaitForKeyRelease;
                            break;
                        }
                    }
                    Ok(ProgramCounter::Pause)
                }
                WaitForKeyState::WaitForKeyRelease => {
                    let mut key_pressed = false;
                    for i in 0..=Key::F as u8 {
                        if input.is_pressed(i.try_into().unwrap()) {
                            key_pressed = true;
                            break;
                        }
                    }

                    if !key_pressed {
                        self.wait_for_key_state = WaitForKeyState::None;
                        Ok(ProgramCounter::Next)
                    } else {
                        Ok(ProgramCounter::Pause)
                    }
                }
                WaitForKeyState::None => Ok(ProgramCounter::Next),
            }
        } else {
            Ok(ProgramCounter::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::graphics::Graphics;
    use crate::traits::GraphicsBuffer;

    use super::FLAG_REGISTER;
    use super::{Chip8, ProgramCounter};

    fn create_chip8(opcode: u16) -> Chip8<Graphics> {
        let graphics = Graphics::new();
        let (_, timer_rx) = mpsc::channel();
        let mut chip8 = Chip8::new(graphics, timer_rx);
        chip8.opcode = opcode;
        chip8
    }

    /// Tests the arithmetic operations of the Chip8 such as addition,
    /// subtraction, multiplication, division, and bitwise operations.
    /// `name` is the name of the test, `test_fn` is the function to be
    /// tested, and `values` is a tuple containing the values that the test
    /// uses, in this order: the opcode, the initial value in register "x", the
    /// initial value in register "y", the final value in register "x", and
    /// the expected value of the carry register.
    macro_rules! test_arithmetic {
        ($($name:ident: ($test_fn:ident, $values:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    let (opcode, reg1_start_val, reg2_start_val, reg1_end, carry) = $values;
                    let mut chip8 = create_chip8(opcode);
                    let (x, y) = chip8.get_regs_x_y();

                    chip8.registers[x] = reg1_start_val;
                    chip8.registers[y] = reg2_start_val;

                    let result = chip8.$test_fn();
                    assert_eq!(chip8.registers[x], reg1_end);
                    assert_eq!(chip8.registers[FLAG_REGISTER], carry);
                    assert_eq!(result, Ok(ProgramCounter::Next));
                }
            )*
        }
    }

    #[test]
    fn test_0x00e0() {
        let mut chip8 = create_chip8(0x00e0);
        // Draw the first sprite digit - digits are loaded starting at 0x0 and are all 5 bytes tall
        chip8.graphics.draw(0, 0, 5, 0, &chip8.memory);

        let pc_op = chip8.opcode_0x0yyy();

        assert_eq!(pc_op, Ok(ProgramCounter::Next));

        let screen = chip8.graphics.buffer();

        for i in screen {
            for j in i {
                assert_eq!(*j, 0);
            }
        }
    }

    #[test]
    fn test_regs_x_y() {
        let chip8 = create_chip8(0x0FA0);

        let (x, y) = chip8.get_regs_x_y();
        assert_eq!(x, 0xF);
        assert_eq!(y, 0xA);
    }

    #[test]
    fn test_bcd() {
        let mut chip8 = create_chip8(0xF133);
        let (x, _) = chip8.get_regs_x_y();

        chip8.registers[x] = 123;
        chip8.ir = 0x500;

        let result = chip8.opcode_0xfyyy();

        assert_eq!(result, Ok(ProgramCounter::Next));
        assert_eq!(chip8.memory[chip8.ir as usize], 1);
        assert_eq!(chip8.memory[chip8.ir as usize + 1], 2);
        assert_eq!(chip8.memory[chip8.ir as usize + 2], 3);
    }

    #[test]
    fn test_copy_to_mem() {
        let mut chip8 = create_chip8(0xF555);

        for i in 0..=5 {
            chip8.registers[i] = (i + 1) as u8;
        }

        chip8.ir = 0x500;

        let result = chip8.opcode_0xfyyy();

        assert_eq!(result, Ok(ProgramCounter::Next));
        assert_eq!(chip8.memory[chip8.ir as usize], 1);
        assert_eq!(chip8.memory[chip8.ir as usize + 1], 2);
        assert_eq!(chip8.memory[chip8.ir as usize + 2], 3);
        assert_eq!(chip8.memory[chip8.ir as usize + 3], 4);
        assert_eq!(chip8.memory[chip8.ir as usize + 4], 5);
        assert_eq!(chip8.memory[chip8.ir as usize + 5], 6);
    }

    #[test]
    fn test_copy_from_mem() {
        let mut chip8 = create_chip8(0xF565);

        chip8.ir = 0x500;

        for i in 0..=5 {
            chip8.memory[chip8.ir as usize + i] = (i + 1) as u8;
        }

        let result = chip8.opcode_0xfyyy();

        assert_eq!(result, Ok(ProgramCounter::Next));

        assert_eq!(chip8.registers[0], 1);
        assert_eq!(chip8.registers[1], 2);
        assert_eq!(chip8.registers[2], 3);
        assert_eq!(chip8.registers[3], 4);
        assert_eq!(chip8.registers[4], 5);
        assert_eq!(chip8.registers[5], 6);
    }

    #[test]
    fn test_1nnn_opcode() {
        let mut chip8 = create_chip8(0x1200);
        chip8.pc = 0x300;

        let result = chip8.opcode_0x1yyy();
        assert_eq!(result, Ok(ProgramCounter::Set(0x200)));
    }

    #[test]
    fn test_2nnn_opcode() {
        let mut chip8 = create_chip8(0x2300);
        let result = chip8.opcode_0x2yyy();

        assert_eq!(result, Ok(ProgramCounter::Set(0x300)));
        assert_eq!(chip8.stack[0], 0x202);
        assert_eq!(chip8.sp, 1);
    }

    macro_rules! test_skip_value_opcodes {
        ($($name:ident: ($test_fn:ident, $values:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    let (opcode, reg_start_val, pc_operation) = $values;
                    let mut chip8 = create_chip8(opcode);
                    let (x, _) = chip8.get_regs_x_y();

                    chip8.registers[x] = reg_start_val;

                    let result = chip8.$test_fn();
                    assert_eq!(result, pc_operation);
                }
            )*
        }
    }

    // First number is opcode, second is register value, third is
    // expected program counter value
    test_skip_value_opcodes! {
        test_0x3yyy_eq: (opcode_0x3yyy, (0x3012, 0x12, Ok(ProgramCounter::Skip))),
        test_0x3yyy_neq: (opcode_0x3yyy, (0x3012, 0x10, Ok(ProgramCounter::Next))),
        test_0x4yyy_eq: (opcode_0x4yyy, (0x3012, 0x12, Ok(ProgramCounter::Next))),
        test_0x4yyy_neq: (opcode_0x4yyy, (0x3012, 0x10, Ok(ProgramCounter::Skip))),

    }

    macro_rules! test_skip_register_opcodes {
        ($($name:ident: ($test_fn:ident, $values:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    let (opcode, reg1_start_val, reg2_start_val, pc_operation) = $values;
                    let mut chip8 = create_chip8(opcode);
                    let (x, y) = chip8.get_regs_x_y();

                    chip8.registers[x] = reg1_start_val;
                    chip8.registers[y] = reg2_start_val;

                    let result = chip8.$test_fn();
                    assert_eq!(result, pc_operation);
                }
            )*
        }
    }

    test_skip_register_opcodes! {
        test_0x3xyy_eq: (opcode_0x3yyy, (0x3110, 0x10, 0x10, Ok(ProgramCounter::Skip))),
        test_0x3xyy_neq: (opcode_0x3yyy, (0x3120, 0x10, 0x10, Ok(ProgramCounter::Next))),
        test_0x4xyy_eq: (opcode_0x4yyy, (0x3110, 0x10, 0x10, Ok(ProgramCounter::Next))),
        test_0x4xyy_neq: (opcode_0x4yyy, (0x3120, 0x10, 0x10, Ok(ProgramCounter::Skip))),
        test_0x5yyy_eq: (opcode_0x5yyy, (0x5120, 0x10, 0x10, Ok(ProgramCounter::Skip))),
        test_0x5yyy_neq: (opcode_0x5yyy, (0x5120, 0x11, 0x10, Ok(ProgramCounter::Next))),
        test_0x9yyy_eq: (opcode_0x9yyy, (0x5120, 0x10, 0x10, Ok(ProgramCounter::Next))),
        test_0x9yyy_neq: (opcode_0x9yyy, (0x5120, 0x11, 0x10, Ok(ProgramCounter::Skip))),
    }

    #[test]
    fn test_0x6yyy_opcode() {
        let mut chip8 = create_chip8(0x6120);
        let (x, _) = chip8.get_regs_x_y();

        chip8.registers[x] = 0;
        let result = chip8.opcode_0x6yyy();

        assert_eq!(chip8.registers[1], 0x20);
        assert_eq!(result, Ok(ProgramCounter::Next));
    }

    #[test]
    fn test_0x7yyy_opcode() {
        let mut chip8 = create_chip8(0x7120);
        let (x, _) = chip8.get_regs_x_y();

        chip8.registers[x] = 0x10;
        let result = chip8.opcode_0x7yyy();

        assert_eq!(chip8.registers[1], 0x30);
        assert_eq!(result, Ok(ProgramCounter::Next));
    }

    #[test]
    fn test_0xayyy() {
        let mut chip8 = create_chip8(0xA120);
        let result = chip8.opcode_0xayyy();

        assert_eq!(chip8.ir, 0x120);
        assert_eq!(result, Ok(ProgramCounter::Next));
    }

    #[test]
    fn test_0xbyyy() {
        let mut chip8 = create_chip8(0xB120);
        chip8.registers[0] = 0xFF;

        let result = chip8.opcode_0xbyyy();

        assert_eq!(result, Ok(ProgramCounter::Set(0xFF + 0x120)));
    }

    // First number is register A, second is register B
    test_arithmetic! {
        test_store: (opcode_0x8yyy, (0x8AB0, 1, 2, 2, 0)),

        test_or_1_1: (opcode_0x8yyy, (0x8AB1, 1, 1, 1, 0)),
        test_or_0_0: (opcode_0x8yyy, (0x8AB1, 0, 0, 0, 0)),
        test_or_0_1: (opcode_0x8yyy, (0x8AB1, 0, 1, 1, 0)),
        test_or_1_0: (opcode_0x8yyy, (0x8AB1, 1, 0, 1, 0)),

        test_and_1_1: (opcode_0x8yyy, (0x8AB2, 1, 1, 1, 0)),
        test_and_0_0: (opcode_0x8yyy, (0x8AB2, 0, 0, 0, 0)),
        test_and_0_1: (opcode_0x8yyy, (0x8AB2, 0, 1, 0, 0)),
        test_and_1_0: (opcode_0x8yyy, (0x8AB2, 1, 0, 0, 0)),

        test_xor_1_1: (opcode_0x8yyy, (0x8AB3, 1, 1, 0, 0)),
        test_xor_0_0: (opcode_0x8yyy, (0x8AB3, 0, 0, 0, 0)),
        test_xor_0_1: (opcode_0x8yyy, (0x8AB3, 0, 1, 1, 0)),
        test_xor_1_0: (opcode_0x8yyy, (0x8AB3, 1, 0, 1, 0)),

        test_add_1_1: (opcode_0x8yyy, (0x8AB4, 1, 1, 2, 0)),
        test_add_254_3: (opcode_0x8yyy, (0x8AB4, 254, 3, 1, 1)),

        test_sub_1_1: (opcode_0x8yyy, (0x8AB5, 1, 1, 0, 1)),
        test_sub_2_1: (opcode_0x8yyy, (0x8AB5, 2, 1, 1, 1)),
        test_sub_2_3: (opcode_0x8yyy, (0x8AB5, 2, 3, 255, 0)),
        test_sub_v3_vf_1: (opcode_0x8yyy, (0x83F5, 5, 5, 0, 1)),
        test_sub_v3_vf_2: (opcode_0x8yyy, (0x83F5, 5, 6, 255, 0)),
        test_sub_v3_vf_3: (opcode_0x8yyy, (0x83F5, 5, 4, 1, 1)),

        test_shr_0: (opcode_0x8yyy, (0x8AB6, 0, 0, 0, 0)),
        test_shr_1: (opcode_0x8yyy, (0x8AB6, 1, 0, 0, 1)),
        test_shr_2: (opcode_0x8yyy, (0x8AB6, 2, 0, 1, 0)),
        test_shr_3: (opcode_0x8yyy, (0x8AB6, 3, 0, 1, 1)),

        test_subn_1_1: (opcode_0x8yyy, (0x8AB7, 1, 1, 0, 1)),
        test_subn_1_2: (opcode_0x8yyy, (0x8AB7, 1, 2, 1, 1)),
        test_subn_2_1: (opcode_0x8yyy, (0x8AB7, 2, 1, 255, 0)),
        test_subn_v3_vf: (opcode_0x8yyy, (0x83F7, 5, 4, 255, 0)),

        test_shl_0: (opcode_0x8yyy, (0x8ABE, 0, 0, 0, 0)),
        test_shl_1: (opcode_0x8yyy, (0x8ABE, 1, 0, 2, 0)),
        test_shl_2: (opcode_0x8yyy, (0x8ABE, 2, 0, 4, 0)),
        test_shl_3: (opcode_0x8yyy, (0x8ABE, 128, 0, 0, 1)),
        test_shl_4: (opcode_0x8yyy, (0x8ABE, 129, 0, 2, 1)),
    }
}
