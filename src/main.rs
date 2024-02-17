mod drivers;
use chip8::{
    chip8::Chip8,
    graphics::Graphics,
    input::{InputUpdate, SdlInput},
    timer::TimerOperation,
    traits::Display,
    Quirks,
};
use measurements::Frequency;

use std::{process, sync::mpsc, thread, time::Duration};

use drivers::{RomDriver, SdlAudioDriver, SdlDisplayDriver};

fn main() -> Result<(), String> {
    let (timer_tx, timer_rx) = mpsc::channel();
    let (input_tx, input_rx) = mpsc::channel();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(17));
        timer_tx.send(TimerOperation::Decrement(1)).unwrap();
    });

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(12));
        input_tx.send(()).unwrap();
    });

    // TODO: replace with clap later
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("missing path to ROM".to_string());
    }

    let sdl_context = sdl2::init()?;
    let mut display = SdlDisplayDriver::new(&sdl_context);
    let audio = SdlAudioDriver::new(&sdl_context);
    let rom = RomDriver::new(&args[1]);
    let mut input = SdlInput::new(&sdl_context, input_rx);
    let graphics = Graphics::new();
    let mut chip8 = Chip8::new(graphics, timer_rx, Quirks::default());

    // Start with 500Hz, make this adjustable later
    let chip8_freq = Frequency::from_hertz(800.into());
    let sleep_time = chip8_freq.as_period();

    match chip8.load_rom(&rom) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

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
