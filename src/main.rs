use std::error::Error;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

mod engine;
use crate::engine::{Chip8Engine, DISPLAY_HEIGHT, DISPLAY_SIZE, DISPLAY_WIDTH};

const FPS: u32 = 5;
const DISPLAY_FPS: u32 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "./src/IBM Logo.ch8";
    // TODO(ben): While passing in a file name alone is very nice, we may want to pass in a
    // "reader" of some sort. That can be a reader to a file name, a in-place string, or something
    // along those lines.
    let mut chip_8 = Chip8Engine::new();
    chip_8.load(file_name)?;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::RGBA8888,
            DISPLAY_WIDTH as u32,
            DISPLAY_HEIGHT as u32,
        )
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let display_interval = Duration::from_secs_f64((1 / DISPLAY_FPS).into());
    let mut last_time = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => todo!("build this with chip_8.press()???"),
                _ => {}
            }
        }

        // Draw the Display at 60Hz
        // TODO(ben): refactor. I believe it's correct to leave this "draw" event within the main
        // loop (a.k.a., do not add a "chip_8.draw()". It should not be part of the engine.
        // Instead, we can read the Engine Display at any time and draw what that should look like
        // when we would like.
        if last_time.elapsed() >= display_interval {
            let mut pixels: [u8; DISPLAY_SIZE * 4] = [0; DISPLAY_SIZE * 4];
            let white = 0xFFFFFFFF_u32.to_be_bytes();
            let black = 0x00000000_u32.to_be_bytes();

            for (i, &pixel) in chip_8.display.iter().enumerate() {
                let offset = i * 4;
                if pixel == 0 {
                    pixels[offset..offset + 4].copy_from_slice(&black);
                } else {
                    pixels[offset..offset + 4].copy_from_slice(&white);
                }
            }

            texture.update(None, &pixels, DISPLAY_WIDTH * 4).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            // Reset timer at the end of the loop.
            last_time = Instant::now();
        }

        chip_8.tick()?;

        // Run the Emulator at 500Ticks/Second. For Now.
        // should I be doing this?
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
    Ok(())
}
