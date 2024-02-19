mod drivers;
use chip8::{chip8::Chip8, graphics::Graphics, timer::TimerOperation, traits::Display, QuirksBuilder};
use clap::Parser;
use measurements::Frequency;

use std::{process, sync::mpsc, thread, time::Duration};

use drivers::{InputUpdate, RomDriver, SdlAudioDriver, SdlDisplayDriver, SdlInput};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Chip 8 ROM to launch
    rom: String,

    /// Quirk: hould the `AND`, `OR`, and `XOR` instructions reset the `VF` register?
    #[arg(long, default_value_t = true)]
    q_reset_vf: bool,

    /// Quirk: should the `Fx55` and `Fx65` opcodes increment the index register?
    /// Games from the 1970s and 1980s might rely on it being incremented.
    /// Modern games might rely on it not being incremented.
    #[arg(long, default_value_t = true)]
    q_increment_ir: bool,

    /// Quirk: should register `VX` be set to the value of register `VY` before shifting?
    /// Modern games might require this to be false.
    #[arg(long, default_value_t = true)]
    q_use_vy_in_shift: bool,

    /// Quirk: allow using registers in `0xBnnn` instruction? Interprets `0xB` instructions
    /// as `0xBXnn`, where `X` is the register to use as part of the jump, i.e.
    /// `VX + nn` instead of `V0 + nnn`.
    #[arg(long, default_value_t = false)]
    q_use_vx_in_jump: bool,

    /// Quirk: clip the drawings that extend past the screen? Otherwise wraps them and
    /// draws them on the other side.
    #[arg(long, default_value_t = true)]
    q_clipping: bool,
}

fn main() -> Result<(), String> {
    // TODO: replace with clap later
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

    let mut chip8 = Chip8::new(graphics, timer_rx, quirks);

    // Start with 500Hz, make this adjustable later
    let chip8_freq = Frequency::from_hertz(800.into());
    let sleep_time = chip8_freq.as_period();

    match chip8.load_rom(&rom) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(17));
        timer_tx.send(TimerOperation::Decrement(1)).unwrap();
    });

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(12));
        input_tx.send(()).unwrap();
    });

    while let InputUpdate::Continue = input.update() {
        let output = chip8.emulate_cycle(input.input()).unwrap();

        display.draw(output.graphics.buffer());

        if output.sound_on {
            audio.start_buzzer();
        } else {
            audio.stop_buzzer();
        }

        thread::sleep(sleep_time);
    }

    process::exit(0);
}
