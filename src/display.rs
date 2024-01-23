use sdl2::{pixels, rect::Rect, render::Canvas, video::Window};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

const SCALE_FACTOR: u16 = 20;
const DISPLAY_WIDTH: u16 = SCREEN_WIDTH * SCALE_FACTOR;
const DISPLAY_HEIGHT: u16 = SCREEN_HEIGHT * SCALE_FACTOR;
const TITLE: &str = "Chip 8";

/// The window that displays the Chip 8 buffer to the screen.
pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    /// Creates a new display window and clears it to black.
    pub fn new(sdl_context: &sdl2::Sdl) -> Display {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(TITLE, DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }

    /// Draws the specified `buffer`. The buffer is expected to be
    /// made up of `1`s and `0`s. `1`s are drawn as white and `0`s
    /// are drawn as black.
    pub fn draw(&mut self, buffer: &Vec<u8>) {
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                let x = row * SCALE_FACTOR;
                let y = col * SCALE_FACTOR;

                let val = buffer[(row * SCREEN_WIDTH + col) as usize];
                let color = pixels::Color::RGB(val * 255, val * 255, val * 255);

                self.canvas.set_draw_color(color);
                let _ = self.canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    SCALE_FACTOR as u32,
                    SCALE_FACTOR as u32,
                ));
            }

            self.canvas.present();
        }
    }
}
