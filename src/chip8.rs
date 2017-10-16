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
    screen: Vec<u8>,
    delay_timer: u8,
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
const SCREEN_SIZE: u16 = 64 * 32;
const MEMORY_SIZE: u8 = 4096;
const STACK_SIZE: u16 = 16;

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
            screen: vec![0; SCREEN_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            stack: vec![0; STACK_SIZE],
            sp: 0,
        }
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = (memory[pc] << 8) | memory[pc + 1];

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
                    self.pc = stack[sp];
                    // Restore stack
                    self.sp -= 1;
                },
            },

            // 0x1adr - Jump to location adr
            0x1000 => {
                pc = self.opcode & 0x0FFF;
            },

            // 0x2adr - Call subroutine at adr
            0x2000 => {
                // Put program counter on stack and then jump to subroutine
                // location
                sp += 1;
                self.stack[sp] = pc;
                pc = 0x0FFF;
            },

            // 3xkk - SE Vx, byte
            // Skip next instruction if Vx == kk
            0x3000 => {
                // Get register value and constant
                let register_val = self.registers[self.opcode & 0x0F00];
                let comp_val = self.opcode & 0x00FF;

                // If equal, skip next instruction (increment program
                // counter by 2)
                if register_val == comp_val {
                    pc += 2;
                }
            },

            // 4xkk - SNE Vx, byte
            // Skip next instruction if Vx != kk
            0x4000 => {
                // Get register value and constant
                let register_val = self.registers[self.opcode & 0x0F00];
                let comp_val = self.opcode & 0x00FF;

                // If not equal, skip next instruction (increment program
                // counter by 2)
                if register_val != comp_val {
                    pc += 2;
                }
            },

            // 5xy0 - SE Vx, Vy
            // Skip next instruction if Vx == Vy
            0x5000 => {
                // Get value in register Vx and Vy
                let vx_val = self.registers[self.opcode & 0x0F00];
                let vy_val = self.registers[self.opcode & 0x00F0];

                // If values are equal, skip next instruction (increment
                // program counter by 2)
                if vx_val == vy_val {
                    pc += 2;
                }
            },

            // 6xkk - LD Vx, byte
            // Set Vx = kk
            0x6000 => {
                // Get value and register number
                let val = self.opcode & 0x00FF;
                let x = self.opcode & 0x0F00;

                // Set register to value
                registers[x] = val;
                pc += 2;
            },

            // 7xkk - ADD Vx, byte
            // Set Vx = Vx + kk
            0x7000 => {
                // Get value and register
                let val = self.opcode & 0x00FF;
                let x = self.opcode & 0x0F00;

                registers[x] += val;
                pc += 2;
            }

            // Multiple instructions start with 8xy so they have to be matched
            // again
            0x8000 => { 

                // Utility function to return the number of registers x and y.
                fn get_regs_x_y() -> (u8, u8) {
                    return (self.opcode & 0x0F00, self.opcode & 0x00F0);
                }

                match self.opcode & 0x000F {
                    // 8xy0 - LD Vx, Vy
                    // Set Vx = Vy
                    0x0000 => {
                        let (x, y) = get_regs_x_y();

                        registers[x] += registers[y];
                        pc += 2;
                    },

                    // 8xy1 - OR Vx, Vy
                    // Perform bitwise OR on Vx and Vy and store result in Vx.
                    0x0001 => {
                        let (x, y) = get_regs_x_y();

                        registers[x] |= registers[y];
                        pc += 2;
                    },

                    // 8xy2 - AND Vx, Vy
                    // Perform bitwise AND on Vx and Vy and store result in Vx.
                    0x0002 => {
                        let (x, y) = get_regs_x_y();

                        registers[x] &= registers[y];
                        pc += 2;
                    },

                    // 8xy3 - XOR Vx, Vy
                    // Performs bitwise XOR on Vx and Vy and stores result in Vx.
                    0x0003 => {
                        let (x, y) = get_regs_x_y();

                        registers[x] ^= registers[y];
                        pc += 2;
                    },

                    // 8xy4 - ADD Vx, Vy
                    // Vx = Vx + Vy, set VF = carry
                    // If the result of Vx and Vy is greater than 8 bits (255)
                    // VF is set to 1, otherwise it's set to 0
                    0x0004 => {
                        let (x, y) = get_regs_x_y();

                        if registers[x] > 0xFF - registers[y] {
                            registers[0xF] = 1;
                        }
                        else {
                            registers[0xF] = 0;
                        }

                        registers[x] += registers[y];
                        pc += 2;
                    },

                    // 8xy5 - SUB Vx, Vy
                    // Vx= Vx - Vy, set VF = NOT borrow
                    // If Vx > Vy, then VF is set to 1, otherwise 0
                    0x0005 => {
                        let (x, y) = get_regs_x_y();

                        if registers[x] > registers[y] {
                            registers[0xF] = 1;
                        }
                        else {
                            registers[0xF] = 0;
                        }

                        registers[x] -= registers[y];
                        pc += 2;
                    },

                    // 8xy6 - SHR Vx {, Vy}
                    // Set Vx = Vx SHR 1
                    // If least significant bit of Vx is 1, then VF is set to 1,
                    // otherwise 0. Then Vx is divided by 2
                    0x0006 => {
                        let (x, _) = get_regs_x_y();

                        if registers[x] & 0x1 == 1 {
                            registers[0xF] = 1;
                        }
                        else {
                            registers[0xF] = 0;
                        }

                        registers[x] >>= 2;
                        pc += 2;
                    },

                    // 8xy7 - SUBN Vx, Vy
                    // Set Vx = Vy - Vx, set VF = NOT borrow
                    // If Vy > Vx, then VF = 1, otherwise VF = 0.
                    0x0007 => {
                        let (x, y) = get_regs_x_y();

                        if registers[y] > registers[x] {
                            registers[0xF] = 1;
                        }
                        else {
                            registers[0xF] = 0;
                        }

                        registers[x] = registers[y] - registers[x];
                        pc += 2;
                    },

                    // 8xyE - SHL Vx {, Vy}
                    // Set Vx = Vx SHL 1
                    // If most significant bit of Vx is 1, set VF to 1, otherwise 0.
                    0x000E => {
                        let (x, _) = get_regs_x_y();

                        if registers[x] & 0x1 == 1 {
                            registers[0xF] = 1;
                        }
                        else {
                            registers[0xF] = 0;
                        }

                        registers[x] <<= 2;
                        pc += 2;
                    },
                }
            },
        }
    }
}
