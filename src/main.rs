mod drivers;
use chip8::{
    chip8::Chip8,
    graphics::Graphics,
    input::{InputUpdate, SdlInput},
    timer::TimerOperation,
    traits::Display,
};

use std::{sync::mpsc, thread, time::Duration};

use drivers::{RomDriver, SdlAudioDriver, SdlDisplayDriver};

fn main() -> Result<(), String> {
    let (timer_tx, timer_rx) = mpsc::channel();

    let timer_thread_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(17));
        timer_tx.send(TimerOperation::Decrement(1)).unwrap();
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
    let mut input = SdlInput::new(&sdl_context);
    let graphics = Graphics::new();
    let mut chip8 = Chip8::new(graphics, timer_rx);

    match chip8.load_rom(&rom) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    while let InputUpdate::Continue = input.update() {
        let output = chip8.emulate_cycle(input.input());

        display.draw(output.graphics.buffer());

        if output.sound_on {
            audio.start_buzzer();
        } else {
            audio.stop_buzzer();
        }
    }

    timer_thread_handle.join().unwrap();

    Ok(())
}
