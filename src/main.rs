mod drivers;
use clap::{ArgAction, Parser};
use measurements::Frequency;
use wheat::{
    chip8::Chip8, graphics::Graphics, timer::TimerOperation, traits::Display, DebugOptionsBuilder,
    QuirksBuilder,
};

use std::{process, sync::mpsc, thread, time::Duration};

use drivers::{InputUpdate, RomDriver, SdlAudioDriver, SdlDisplayDriver, SdlInput};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Chip 8 ROM to launch
    rom: String,

    /// Frequency (in Hz) for the Chip 8 CPU to run at.
    #[arg(short, long, default_value_t = 800)]
    freq_cpu: u32,

    /// Frequency (in Hz) for the input system to scan new keycodes.
    #[arg(long, default_value_t = 12)]
    freq_input: u32,

    /// Frequency (in Hz) for the timers. It is not recommended to change it from
    /// the default value.
    #[arg(long, default_value_t = 60)]
    freq_timer: u32,

    /// Quirk: hould the `AND`, `OR`, and `XOR` instructions reset the `VF` register?
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    q_reset_vf: bool,

    /// Quirk: should the `Fx55` and `Fx65` opcodes increment the index register?
    /// Games from the 1970s and 1980s might rely on it being incremented.
    /// Modern games might rely on it not being incremented.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    q_increment_ir: bool,

    /// Quirk: should register `VX` be set to the value of register `VY` before shifting?
    /// Modern games might require this to be false.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    q_use_vy_in_shift: bool,

    /// Quirk: allow using registers in `0xBnnn` instruction? Interprets `0xB` instructions
    /// as `0xBXnn`, where `X` is the register to use as part of the jump, i.e.
    /// `VX + nn` instead of `V0 + nnn`.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    q_use_vx_in_jump: bool,

    /// Quirk: clip the drawings that extend past the screen? Otherwise wraps them and
    /// draws them on the other side.
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    q_clipping: bool,

    /// Print opcodes as they're interpreted.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    print_opcodes: bool,

    /// Dump the graphics buffer after every draw opcode.
    #[arg(long, default_value_t = false, action = ArgAction::Set)]
    dump_graphics: bool,
}

fn freq_to_time(hertz: f64) -> Duration {
    let freq = Frequency::from_hertz(hertz);
    freq.as_period()
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let (timer_tx, timer_rx) = mpsc::channel();
    let (input_tx, input_rx) = mpsc::channel();

    let sdl_context = sdl2::init()?;
    let mut display = SdlDisplayDriver::new(&sdl_context);
    let audio = SdlAudioDriver::new(&sdl_context);
    let rom = RomDriver::new(&args.rom);
    let mut input = SdlInput::new(&sdl_context, input_rx);
    let graphics = Graphics::new();

    let quirks = QuirksBuilder::default()
        .reset_vf(args.q_reset_vf)
        .increment_ir(args.q_increment_ir)
        .use_vy_in_shift(args.q_use_vy_in_shift)
        .use_vx_in_jump(args.q_use_vx_in_jump)
        .clipping(args.q_clipping)
        .build()
        .unwrap();

    let options = DebugOptionsBuilder::default()
        .print_opcodes(args.print_opcodes)
        .dump_graphics(args.dump_graphics)
        .build()
        .unwrap();

    let mut chip8 = Chip8::new(graphics, timer_rx, quirks, options);

    let chip8_freq = Frequency::from_hertz(args.freq_cpu.into());
    let emulation_sleep_time = chip8_freq.as_period();

    chip8.load_rom(&rom).map_err(|e| e.to_string())?;

    // Setup separate threads for managing input and timer updates
    let timer_sleep = freq_to_time(args.freq_timer.into());
    let input_sleep = freq_to_time(args.freq_input.into());

    thread::spawn(move || loop {
        thread::sleep(timer_sleep);
        timer_tx.send(TimerOperation::Decrement(1)).unwrap();
    });

    thread::spawn(move || loop {
        thread::sleep(input_sleep);
        input_tx.send(()).unwrap();
    });

    while let InputUpdate::Continue = input.update() {
        let output = chip8.emulate_cycle(input.input()).map_err(|e| e.to_string())?;

        display.draw(output.graphics.buffer());

        if output.sound_on {
            audio.start_buzzer();
        } else {
            audio.stop_buzzer();
        }

        thread::sleep(emulation_sleep_time);
    }

    process::exit(0);
}
