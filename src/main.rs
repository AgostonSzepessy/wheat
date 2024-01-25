mod drivers;

use drivers::SdlDisplayDriver;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let display = SdlDisplayDriver::new(&sdl_context);

    Ok(())
}
