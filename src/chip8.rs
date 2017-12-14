use rand;
use rand::Rng;

use graphics::Graphics;
use input::Input;

#[derive(Debug)]
pub struct Chip8 {
    /// Current opcode
    opcode: u16,
    /// The system has 4096 bytes of memory.
    memory: Vec<u8>,
    /// The index register
    ir: u16,
    /// The program counter
    pc: u16,
    /// Screen that sprites get drawn on. 64x32 pixels
    graphics: Graphics,
    delay_timer: u8,
    registers: Vec<u8>,
    /// When this timer reaches 0, the system's buzzer sounds
    sound_timer: u8,
    /// Function call stack. When a jump is performed, the current location
    /// is pushed on the stack so it can be retrieved later.
    stack: Vec<u16>,
    /// The stack pointer
    sp: u8,
}

// The default address at which the application is loaded at
const APP_LOCATION: u16 = 0x200;

// Total memory available to Chip8
const MEMORY_SIZE: usize = 4096;

// Total size of the stock
const STACK_SIZE: usize = 16;

// Number of registers available
const NUM_REGISTERS: usize = 16;

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

// Throughout the code, Vx refers to the general purpose registers. There are
// 15 general purpose registers from V0 to VE. The 16th register is used to
// represent the carry flag.

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = vec![0; MEMORY_SIZE];

        for i in 0..HEX_DIGITS.len() {
            memory[i] = HEX_DIGITS[i];
        }

        Chip8 {
            opcode: 0,
            memory: memory,
            ir: 0,
            pc: APP_LOCATION,
            graphics: Graphics::new(),
            delay_timer: 0,
            registers: vec![0; NUM_REGISTERS],
            sound_timer: 0,
            stack: vec![0; STACK_SIZE],
            sp: 0,
        }
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8) | self.memory[self.pc as usize + 1] as u16;

        match self.opcode & 0xF000 {
            // Opcode starts with 0x0
            0x0000 => {
                self.opcode_0x0yyy();
            },

            // Opcode starts with 0x1
            0x1000 => {
                self.opcode_0x1yyy();
            },

            // Opcode starts with 0x2
            0x2000 => {
                self.opcode_0x2yyy();
            },

            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx == kk
            0x3000 => {
                self.opcode_0x3yyy();
            },

            // Opcodes that start with 0x4
            0x4000 => {
                self.opcode_0x4yyy();
            },

            // Opcodes that start with 0x5
            0x5000 => {
                self.opcode_0x5yyy();
            },

            // Opcodes that start with 0x6
            0x6000 => {
                self.opcode_0x6yyy();
            },

            // Opcodes that start with 0x7
            0x7000 => {
                self.opcode_0x7yyy();
            }

            // Opcodes that start with 0x8
            0x8000 => { 
                self.opcode_0x8yyy();
            },

            // Opcodes that start with 0x9
            0x9000 => {
                self.opcode_0x9yyy();
            },

            // Opcodes that start with 0xA
            0xA000 => {
                self.opcode_0xayyy();
            },

            // Opcodes that start with 0xB
            0xB000 => {
                self.opcode_0xbyyy();
            },

            // Cxkk - RND, byte
            // Set Vx = random byte AND kk
            // Interpreter generates a random number between 0 and 255, which
            // is then ANDed with kk and the result is stored in Vx.
            0xC000 => {
                self.opcode_0xcyyy();
            },

            0xD000 => {
                self.opcode_0xdyyy();
            },

            _ => {
                self.unknown_opcode();
            }
        }
    }

    // Utility function to return the number of registers x and y.
    fn get_regs_x_y(&self) -> (usize, usize) {
        return (((self.opcode & 0x0F00) >> 8) as usize, ((self.opcode & 0x00F0) >> 4) as usize);
    }

    fn unknown_opcode(&mut self) {
        println!("unknown opcode: {:X}", self.opcode);
        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0x0.
    fn opcode_0x0yyy(&mut self) {
        match self.opcode & 0x00FF {
            // Clear the screen
            0x00E0 => {
                self.graphics.clear();
            },
            // Return from subroutine
            0x00EE => {
                // Restore program counter to previous location on stack
                // before subroutine was called
                self.pc = self.stack[self.sp as usize];
                // Restore stack
                self.sp -= 1;
            },

            // No other opcodes start with 0x0
            _ => {
                self.unknown_opcode();
            },
        }
    }

    /// Takes care of opcodes that start with 0x1.
    fn opcode_0x1yyy(&mut self) {
        // Only 1 opcode that starts with 0x1: 0x1nnn
        // 0x1nnn - Jump to location nnn
        self.pc = self.opcode & 0x0FFF;
    }

    /// Takes care of opcodes that start with 0x2.
    fn opcode_0x2yyy(&mut self) {
        // 0x2adr - Call subroutine at adr
        // Put program counter on stack and then jump to subroutine
        // location
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = self.opcode & 0x0FFF;
    }

    /// Takes care of opcodes that start with 0x3.
    fn opcode_0x3yyy(&mut self) {
        // 3xkk - SE Vx, byte
        // Skip next instruction if Vx == kk
        
        // Get register value and constant
        let (x, _) = self.get_regs_x_y();
        let register_val = self.registers[x];
        let comp_val = (self.opcode & 0x00FF) as u8;

        // If equal, skip next instruction (increment program
        // counter by 2)
        if register_val == comp_val {
            self.pc += 2;
        }

        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0x4.
    fn opcode_0x4yyy(&mut self) {
        // 4xkk - SNE Vx, byte
        // Skip next instruction if Vx != kk

        // Get register value and constant
        let (x, _) = self.get_regs_x_y();
        let register_val = self.registers[x];
        let comp_val = (self.opcode & 0x00FF) as u8;

        // If not equal, skip next instruction (increment program
        // counter by 2)
        if register_val != comp_val {
            self.pc += 2;
        }

        self.pc += 2;

    }

    /// Takes care of opcodes that start with 0x5.
    fn opcode_0x5yyy(&mut self) {
        // 5xy0 - SE Vx, Vy
        // Skip next instruction if Vx == Vy
        let (x, y) = self.get_regs_x_y();
        let vx_val = self.registers[x];
        let vy_val = self.registers[y];

        // If values are equal, skip next instruction (increment
        // program counter by 2)
        if vx_val == vy_val {
            self.pc += 2;
        }

        self.pc += 2;

    }

    /// Takes care of opcodes that start with 0x6.
    fn opcode_0x6yyy(&mut self) {
        // 6xkk - LD Vx, byte
        // Set Vx = kk
        let val = (self.opcode & 0x00FF) as u8;
        let (x, _) = self.get_regs_x_y();

        // Set register to value
        self.registers[x] = val;
        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0x7.
    fn opcode_0x7yyy(&mut self) {
        // 7xkk - ADD Vx, byte
        // Set Vx = Vx + kk
            // Get value and register
            let val = (self.opcode & 0x00FF) as u8;
            let x = ((self.opcode & 0x0F00) >> 8) as usize;

            self.registers[x] += val;
            self.pc += 2;

    }

    /// Takes care of opcodes that start with 0x8.
    fn opcode_0x8yyy(&mut self) {
        // Last nibble identifies what the opcode does
        match self.opcode & 0x000F {
            // 8xy0 - LD Vx, Vy
            // Set Vx = Vy
            0x0000 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] = self.registers[y];
                self.pc += 2;
            },

            // 8xy1 - OR Vx, Vy
            // Perform bitwise OR on Vx and Vy and store result in Vx.
            0x0001 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] |= self.registers[y];
                self.pc += 2;
            },

            // 8xy2 - AND Vx, Vy
            // Perform bitwise AND on Vx and Vy and store result in Vx.
            0x0002 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] &= self.registers[y];
                self.pc += 2;
            },

            // 8xy3 - XOR Vx, Vy
            // Performs bitwise XOR on Vx and Vy and stores result in Vx.
            0x0003 => {
                let (x, y) = self.get_regs_x_y();

                self.registers[x] ^= self.registers[y];
                self.pc += 2;
            },

            // 8xy4 - ADD Vx, Vy
            // Vx = Vx + Vy, set VF = carry
            // If the result of Vx and Vy is greater than 8 bits (255)
            // VF is set to 1, otherwise it's set to 0
            0x0004 => {
                let (x, y) = self.get_regs_x_y();
                let (val, overflow) = self.registers[x].overflowing_add(self.registers[y]);

                if overflow {
                    self.registers[0xF] = 1;
                }
                else {
                    self.registers[0xF] = 0;
                }

                self.registers[x] = val;
                self.pc += 2;
            },

            // 8xy5 - SUB Vx, Vy
            // Vx= Vx - Vy, set VF = NOT borrow
            // If Vx > Vy, then VF is set to 1, otherwise 0
            0x0005 => {
                let (x, y) = self.get_regs_x_y();

                if self.registers[x] > self.registers[y] {
                    self.registers[0xF] = 1;
                }
                else {
                    self.registers[0xF] = 0;
                }

                let (val, _) = self.registers[x].overflowing_sub(self.registers[y]);
                self.registers[x] = val;
                self.pc += 2;
            },

            // 8xy6 - SHR Vx {, Vy}
            // Set Vx = Vx SHR 1
            // If least significant bit of Vx is 1, then VF is set to 1,
            // otherwise 0. Then Vx is divided by 2
            0x0006 => {
                let (x, _) = self.get_regs_x_y();

                self.registers[0xF] = self.registers[x] & 0x1;

                self.registers[x] >>= 1;
                self.pc += 2;
            },

            // 8xy7 - SUBN Vx, Vy
            // Set Vx = Vy - Vx, set VF = NOT borrow
            // If Vy > Vx, then VF = 1, otherwise VF = 0.
            0x0007 => {
                let (x, y) = self.get_regs_x_y();

                if self.registers[y] > self.registers[x] {
                    self.registers[0xF] = 1;
                }
                else {
                    self.registers[0xF] = 0;
                }

                let (val, _) = self.registers[y].overflowing_sub(self.registers[x]);
                self.registers[x] = val;
                self.pc += 2;
            },

            // 8xyE - SHL Vx {, Vy}
            // Set Vx = Vx SHL 1
            // If most significant bit of Vx is 1, set VF to 1, otherwise 0.
            0x000E => {
                let (x, _) = self.get_regs_x_y();

                self.registers[0xF] = self.registers[x] & 0x1;
                self.registers[x] <<= 1;
                self.pc += 2;
            },

            // No other opcodes start with 0x8
            _ => {
                self.unknown_opcode();
            }
        }
    }

    /// Takes care of opcodes that start with 0x9
    fn opcode_0x9yyy(&mut self) {
        // 9xy0 - SNE Vx, Vy
        // Skip next instruction if Vx != Vy
        let (x, y) = self.get_regs_x_y();

        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }

        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0xA
    fn opcode_0xayyy(&mut self) {
        // Annn - LD I, addr
        // Set I = nnn
        // Get address and set index register
        let val = self.opcode & 0x0FFF;
        self.ir = val;
        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0xB
    fn opcode_0xbyyy(&mut self) {
        // Bnnn - JP V0, nnn
        // Jump to location nnn + V0 (set pc = nnn + V0)
        let val = self.opcode & 0x0FFF;
        self.pc = val + self.registers[0x0] as u16;
    }

    /// Takes care of opcodes that start with 0xC
    fn opcode_0xcyyy(&mut self) {
        // Cxkk - RND, byte
        // Set Vx = random byte AND kk
        // Interpreter generates a random number between 0 and 255, which
        // is then ANDed with kk and the result is stored in Vx.
        let kk: u8 = (self.opcode & 0x00FF) as u8;
        let (x, _) = self.get_regs_x_y();

        let rand_val = rand::thread_rng().gen_range::<u16>(0, 256) as u8;

        self.registers[x] = rand_val & kk;
        self.pc += 2;
    }

    /// Takes care of opcodes that start with 0xD
    fn opcode_0xdyyy(&mut self) {
        // Dxyn - DRW Vx, Vy, nibble
        // Display n-byte sprite starting at memory location I at (Vx, Vy), 
        // set VF = collision
        let flipped = self.graphics.draw(&self.opcode, &self.ir, &self.memory);

        if flipped {
            self.registers[0xF] = 1;
        }
        else {
            self.registers[0xF] = 0;
        }
    }

    /// Takes care of opcodes that are related to input such as checking whether
    /// a key is pressed or not pressed, and waiting until a key is pressed.
    fn handle_input(&mut self, input: &Input) {
        match self.opcode & 0x0001 {
            // Ex9E - SKP Vx
            // Skips the next instruction if the key with the value of Vx is 
            // pressed. If the key corresponding to the value of Vx is currently
            // in the down position, PC is increased by 2.
            0xE => {
                let (x, _) = self.get_regs_x_y();

                if input.is_pressed(&(x as u8)) {
                    self.pc += 2;
                }

                self.pc += 2;
            },

            // Ex9E - SKNP Vx
            // Skip next instruction if key with value Vx is not pressed. If the
            // key with value Vx is not pressed, the program counter is incremented
            // by 2.
            0x1 => {
                let (x, _) = self.get_regs_x_y();

                if !input.is_pressed(&(x as u8)) {
                    self.pc += 2;
                }

                self.pc += 2;
            },

            // Fx0A - LD Vx, K
            // Wait for a key press, store the value of the key in Vx.
            // All execution stops until a key is pressed, then the value
            // of that key is stored in Vx.
            0xA => {
                let (x, _) = self.get_regs_x_y();

                // Loop from 0 to 15 (use 0x10 because `..` is exclusive for the upper
                // range
                for i in 0x0..0x10 {
                    if input.is_pressed(&i) {
                        self.registers[x] = i;
                        self.pc += 2;
                        break;
                    }
                }
            },

            _ => {
                self.unknown_opcode();
            }
        }
    }

    fn opcode_0xfyyy(&mut self) {
        match self.opcode & 0xFF {
            // Fx07 - LD Vx, DT
            // Set Vx = delay timer value.
            // The value of DT is placed into Vx.
            0x07 => {
                let (x, _) = self.get_regs_x_y();
                self.registers[x] = self.delay_timer;
                self.pc += 2;
            },

            // Fx15 - LD DT, Vx
            // Set delay timer = Vx
            // DT is set equal to the value of Vx.
            0x15 => {
                let (x, _) = self.get_regs_x_y();
                self.delay_timer = self.registers[x];
                self.pc += 2;
            },

            // Fx18 - LD ST, Vx
            // Set sound timer = Vx
            // ST is set equal to the value of Vx.
            0x18 => {
                let (x, _) = self.get_regs_x_y();
                self.sound_timer = self.registers[x];
                self.pc += 2;
            },

            // Fx1E - ADD I, Vx
            // Set I = I + Vx
            // The values of I and Vx are added, and the results are stored in I.
            0x1E => {
                let (x, _) = self.get_regs_x_y();
                self.ir += self.registers[x] as u16;
                self.pc += 2;
            },

            // Fx29 - LD F, Vx
            // Set I = location of sprite for digit Vx.
            // The value of I is set to the location for the hexadecimal sprite
            // corresponding to the value of Vx.
            0x29 => {
                let (x, _) = self.get_regs_x_y();
                // Each hex sprite takes up 5 bytes, and they start at address
                // 0x0, so multiplying the value in Vx by 5 will get us the
                // address of the sprite
                self.ir = x as u16 * 5;
                self.pc += 2;
            },

            _ => {

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chip8;

    fn create_chip8(opcode: u16) -> Chip8 {
        let mut chip8 = Chip8::new();
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

                    chip8.$test_fn();
                    assert_eq!(chip8.registers[x], reg1_end);
                    assert_eq!(chip8.registers[0xF], carry);
                    assert_eq!(chip8.pc, 0x202);
                }
            )*
        }
    }

    #[test]
    fn test_regs_x_y() {
        let chip8 = create_chip8(0x0FA0);

        let (x, y) = chip8.get_regs_x_y();
        assert_eq!(x, 0xF);
        assert_eq!(y, 0xA);
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

        test_sub_1_1: (opcode_0x8yyy, (0x8AB5, 1, 1, 0, 0)),
        test_sub_2_1: (opcode_0x8yyy, (0x8AB5, 2, 1, 1, 1)),
        test_sub_2_3: (opcode_0x8yyy, (0x8AB5, 2, 3, 255, 0)),

        test_shr_0: (opcode_0x8yyy, (0x8AB6, 0, 0, 0, 0)),
        test_shr_1: (opcode_0x8yyy, (0x8AB6, 1, 0, 0, 1)),
        test_shr_2: (opcode_0x8yyy, (0x8AB6, 2, 0, 1, 0)),
        test_shr_3: (opcode_0x8yyy, (0x8AB6, 3, 0, 1, 1)),

        test_subn_1_1: (opcode_0x8yyy, (0x8AB7, 1, 1, 0, 0)),
        test_subn_1_2: (opcode_0x8yyy, (0x8AB7, 1, 2, 1, 1)),
        test_subn_2_1: (opcode_0x8yyy, (0x8AB7, 2, 1, 255, 0)),

        test_shl_0: (opcode_0x8yyy, (0x8ABE, 0, 0, 0, 0)),
        test_shl_1: (opcode_0x8yyy, (0x8ABE, 1, 0, 2, 1)),
        test_shl_2: (opcode_0x8yyy, (0x8ABE, 2, 0, 4, 0)),
        test_shl_3: (opcode_0x8yyy, (0x8ABE, 3, 0, 6, 1)),
    }
}
