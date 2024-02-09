mod drivers;
use chip8::{
    chip8::Chip8,
    input::{InputUpdate, SdlInput},
    timer::TimerOperation,
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

    let sdl_context = sdl2::init()?;
    let mut display = SdlDisplayDriver::new(&sdl_context);
    let audio = SdlAudioDriver::new(&sdl_context);
    let rom = RomDriver::new(&args[1]);
    let mut input = SdlInput::new(&sdl_context);

    while let InputUpdate::Continue = input.update() {}

    timer_thread_handle.join().unwrap();

    Ok(())
}
