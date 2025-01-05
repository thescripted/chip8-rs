use std::error::Error;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

mod engine;
use crate::engine::{Chip8Engine, DISPLAY_HEIGHT, DISPLAY_SIZE, DISPLAY_WIDTH};

use clap::Parser;

const CLOCK: u32 = 500; // Hz
const DISPLAY_FPS: u32 = 60;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let file_path = args.file;
    let rom = std::fs::read(file_path)?;

    let mut chip_8 = Chip8Engine::new();
    chip_8.load(&rom);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CHIP-8", 640, 320)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::RGBA8888,
        DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
    )?;

    let mut event_pump = sdl_context.event_pump()?;

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
                } => todo!("build this with chip_8.press()"),
                _ => {}
            }
        }

        // Draw the Display at 60Hz
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

            texture.update(None, &pixels, DISPLAY_WIDTH * 4)?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            // Reset timer at the end of the loop.
            last_time = Instant::now();
        }

        chip_8.tick()?;

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / CLOCK));
    }
    Ok(())
}
