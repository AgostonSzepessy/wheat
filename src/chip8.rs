pub struct Chip8 {
    /// Current opcode
    opcode: u16,
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
    stack: Vec<u8>,
    /// The stack pointer
    sp: u8,
}
