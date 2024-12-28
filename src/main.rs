use std::error::Error;
use std::ops::{Deref, DerefMut};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const FONT_SET: [i32; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// TODO(ben): for simplicity, let's use vectors for arrays and later optimize for stack-allocated
// arrays once I figure out how to use them properly...
#[derive(Debug)]
struct Chip8Engine {
    rom: Vec<u8>,
    memory: Vec<u8>,
    registers: Vec<u8>,
    index_register: u16,
    program_counter: usize, //TODO(ben): should this be u16?
    delay_timer: u8,
    sound_timer: u8,
    display: Display,
    keyboard: u16,
    stack: Vec<u16>,
}

#[derive(Debug)]
struct Display {
    screen: [u8; DISPLAY_WIDTH / 8 * DISPLAY_HEIGHT],
    width: usize,
    height: usize,
}

// TODO(ben): Question this. Is this an abstraction or an indirection?
impl Display {
    fn new() -> Self {
        Display {
            screen: [0x0; DISPLAY_WIDTH / 8 * DISPLAY_HEIGHT],
            width: DISPLAY_WIDTH / 8,
            height: DISPLAY_HEIGHT,
        }
    }
    // TODO(ben): where should modulo operation be? In the callsite or in the function?
    /// write_pixel writes a pixel value at the point x, y, returning the new value of that pixel.
    /// If that point does not exist in the display, it will return none.
    fn write_pixel(&mut self, x: usize, y: usize, value: u8) -> Option<u8> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.screen[x + y * self.height] = value;
            self.get_pixel(x, y)
        }
    }

    /// get_pixel returns a pixel value at the point x, y. If that point does not exist in the
    /// display, it will return none.
    fn get_pixel(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.screen[x + y * self.height])
        }
    }
}

impl Deref for Display {
    type Target = [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8];

    fn deref(&self) -> &Self::Target {
        &self.screen
    }
}

impl DerefMut for Display {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.screen
    }
}

#[derive(Debug)]
struct Opcode {
    x: u8, //TODO(Ben): this is exploration. Is usize better or u16? u8? boolean?
    y: u8,
    n: u8,
    kk: u8,
    addr: u16,
}

impl Chip8Engine {
    pub fn new(rom_file: &str) -> Result<Self, Box<::std::io::Error>> {
        let rom = std::fs::read(rom_file).unwrap();
        // initialize font_set
        let mut memory = vec![0; 2096];
        for (i, &font_byte) in FONT_SET.iter().enumerate() {
            memory[i + 0x50] = font_byte as u8;
        }

        for (i, byte) in rom.iter().enumerate() {
            memory[0x200 + i] = *byte;
        }

        Ok(Chip8Engine {
            rom: rom.to_vec(),
            program_counter: 0x200,
            memory,
            registers: vec![0; 16],
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: Display::new(),
            keyboard: 0,
            stack: Vec::new(), // TODO(ben): sized vec?
        })
    }

    pub fn _run() {
        todo!(
            "Run will handle various operations, from reading flags that user can provide.
            These flags will primarily control what an operation might do, how fast the game
            engine will run, debug controls, etc.

            Run will also initialize various I/O devices such as keyboard control, sounds, display
            and manage the connections with those devices efficiently."
        );
    }

    // Tick handles the fetch/decode/execute loop for a single operation.
    fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        let code = {
            let slice = [
                self.memory[self.program_counter],
                self.memory[self.program_counter + 1],
            ];

            self.program_counter += 2;

            u16::from_be_bytes(slice)
        };

        let opcode = Opcode {
            x: (code & 0x0100 >> 2) as u8,
            y: (code & 0x0011 >> 1) as u8,
            n: (code & 0x0001) as u8,
            kk: (code & 0x0011) as u8,
            addr: (code & 0x0111) as u16,
        };

        match code & 0xF000 {
            0x0000 => match code & 0x00F0 {
                // 0x00E0 - CLS
                0x00E0 => {
                    for pixel in self.display.iter_mut() {
                        *pixel = 0x0;
                    }
                }
                _ => todo!(),
            },

            // TODO(ben): why cast to usize? any alternatives?
            // 0x1nnn - JP addr
            0x1000 => self.program_counter = opcode.addr.into(),

            // 0x6xnn - LD Vx, byte
            0x6000 => self.registers[opcode.x as usize] = opcode.kk,

            // 0x7xnn - ADD Vx, byte
            0x7000 => self.registers[opcode.x as usize] += opcode.kk,

            // 0xAnnn - LD I, addr
            0xA000 => self.index_register = opcode.addr,

            // 0xDxyn - DRW Vx, Vy, nibble
            //
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            // The interpreter reads n bytes from memory, starting at the address stored in I.
            // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
            // Sprites are XORed onto the existing screen.
            // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
            // If the sprite is positioned so part of it is outside the coordinates of the display,
            // it wraps around to the opposite side of the screen.
            0xD000 => {
                // TODO(ben): get a hold of your numeric casting.
                let x = self.registers[opcode.x as usize] % self.display.width as u8;
                let y = self.registers[opcode.y as usize] % self.display.height as u8;

                self.registers[0xF] = 0;

                let start = self.index_register as usize;
                for i in 0..opcode.n {
                    let read = self.memory[start + i as usize];
                    let curr_x = x.into();
                    let curr_y = (y + i) as usize;
                    let current_pixel = self.display.get_pixel(curr_x, curr_y).unwrap_or(0);

                    let pixel_to_draw = read ^ current_pixel;
                    self.display
                        .write_pixel(curr_x, curr_y, pixel_to_draw)
                        .unwrap_or(0);

                    // Collision.
                    //
                    // A ^ B will turn a bit to zero if and only if
                    // A and B both have a bit that was already. therefore
                    // a collision must have occured if A & B is not zero.
                    if read & current_pixel != 0 {
                        self.registers[0xF] = 1;
                    }
                }
            }
            _ => return Err("unknown instruction".into()),
        };

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "./src/IBM Logo.ch8";
    let mut chip_8 = Chip8Engine::new(file_name)?;

    for _ in 0..200 {
        chip_8.tick()?;
    }

    println!("{:?}", chip_8.display);

    Ok(())
}
