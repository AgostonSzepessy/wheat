mod drivers;
use chip8::timer::TimerOperation;

use std::{sync::mpsc, thread, time::Duration};

use drivers::SdlDisplayDriver;

fn main() -> Result<(), String> {
    let (timer_tx, timer_rx) = mpsc::channel();

    let timer_thread_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(17));
        timer_tx.send(TimerOperation::Decrement(1)).unwrap();
    });

    let sdl_context = sdl2::init()?;
    let display = SdlDisplayDriver::new(&sdl_context);

    timer_thread_handle.join().unwrap();

    Ok(())
}
