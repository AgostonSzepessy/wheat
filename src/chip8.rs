use rand;
use rand::Rng;

use graphics::Graphics;

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
    screen: Graphics,
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

/// The default address at which the application is loaded at
const APP_LOCATION: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_REGISTERS: usize = 16;

// Throughout the code, Vx refers to the general purpose registers. There are
// 15 general purpose registers from V0 to VE. The 16th register is used to
// represent the carry flag.

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            opcode: 0,
            memory: vec![0; MEMORY_SIZE],
            ir: 0,
            pc: APP_LOCATION,
            screen: Graphics::new(),
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
            // Opcode starts with 0
            0x0000 => match self.opcode & 0x00FF {
                // Clear the screen
                0x00E0 => {
                    unimplemented!();
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
                }
            },

            // 0x1adr - Jump to location adr
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            },

            // 0x2adr - Call subroutine at adr
            0x2000 => {
                // Put program counter on stack and then jump to subroutine
                // location
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = self.opcode & 0x0FFF;
            },

            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx == kk
            0x3000 => {
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
            },

            // 4xkk - SNE Vx, byte
            // Skip next instruction if Vx != kk
            0x4000 => {
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
            },

            // 5xy0 - SE Vx, Vy
            // Skip next instruction if Vx == Vy
            0x5000 => {
                // Get value in register Vx and Vy
                let (x, y) = self.get_regs_x_y();
                let vx_val = self.registers[x];
                let vy_val = self.registers[y];

                // If values are equal, skip next instruction (increment
                // program counter by 2)
                if vx_val == vy_val {
                    self.pc += 2;
                }

                self.pc += 2;
            },

            // 6xkk - LD Vx, byte
            // Set Vx = kk
            0x6000 => {
                // Get value and register number
                let val = (self.opcode & 0x00FF) as u8;
                let x = (self.opcode & 0x0F00 >> 8) as usize;

                // Set register to value
                self.registers[x] = val;
                self.pc += 2;
            },

            // 7xkk - ADD Vx, byte
            // Set Vx = Vx + kk
            0x7000 => {
                // Get value and register
                let val = (self.opcode & 0x00FF) as u8;
                let x = ((self.opcode & 0x0F00) >> 8) as usize;

                self.registers[x] += val;
                self.pc += 2;
            }

            // Multiple instructions start with 8xy so they have to be matched
            // again
            0x8000 => { 
                match self.opcode & 0x000F {
                    // 8xy0 - LD Vx, Vy
                    // Set Vx = Vy
                    0x0000 => {
                        let (x, y) = self.get_regs_x_y();

                        self.registers[x] += self.registers[y];
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

                        if self.registers[x] > 0xFF - self.registers[y] {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }

                        self.registers[x] += self.registers[y];
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

                        self.registers[x] -= self.registers[y];
                        self.pc += 2;
                    },

                    // 8xy6 - SHR Vx {, Vy}
                    // Set Vx = Vx SHR 1
                    // If least significant bit of Vx is 1, then VF is set to 1,
                    // otherwise 0. Then Vx is divided by 2
                    0x0006 => {
                        let (x, _) = self.get_regs_x_y();

                        if self.registers[x] & 0x1 == 1 {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }

                        self.registers[x] >>= 2;
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

                        self.registers[x] = self.registers[y] - self.registers[x];
                        self.pc += 2;
                    },

                    // 8xyE - SHL Vx {, Vy}
                    // Set Vx = Vx SHL 1
                    // If most significant bit of Vx is 1, set VF to 1, otherwise 0.
                    0x000E => {
                        let (x, _) = self.get_regs_x_y();

                        if self.registers[x] & 0x1 == 1 {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }

                        self.registers[x] <<= 2;
                        self.pc += 2;
                    },

                    // No other opcodes start with 0x8
                    _ => {
                        self.unknown_opcode();
                    }
                }
            },

            // 9xy0 - SNE Vx, Vy
            // Skip next instruction if Vx != Vy
            0x9000 => {
                let (x, y) = self.get_regs_x_y();

                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }

                self.pc += 2;
            },

            // Annn - LD I, addr
            // Set I = addr
            0xA000 => {
                // Get value and set index register
                let val = self.opcode & 0x0FFF;
                self.ir = val;
                self.pc += 2;
            },

            // Bnnn - JP V0, nnn
            // Jump to location nnn + V0 (set pc = nnn + V0)
            0xB000 => {
                let val = self.opcode & 0x0FFF;
                self.pc = val + self.registers[0x0] as u16;
            },

            // Cxkk - RND, byte
            // Set Vx = random byte AND kk
            // Interpreter generates a random number between 0 and 255, which
            // is then ANDed with kk and the result is stored in Vx.
            0xC000 => {
                let kk: u8 = (self.opcode & 0x00FF) as u8;
                let x = ((self.opcode & 0x0F00) >> 8) as usize;

                let rand_val = rand::thread_rng().gen_range::<u16>(0, 256) as u8;

                self.registers[x] = rand_val & kk;
                self.pc += 2;
            },

            0xD000 => {
                unimplemented!();
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

}
