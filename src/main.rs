mod drivers;
use chip8::{input::SdlInput, timer::TimerOperation};

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
    let display = SdlDisplayDriver::new(&sdl_context);
    let audio = SdlAudioDriver::new(&sdl_context);
    let rom = RomDriver::new(&args[1]);
    let input = SdlInput::new(&sdl_context);

    timer_thread_handle.join().unwrap();

    Ok(())
}
