use std::error::Error;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const MEMORY_SIZE: usize = 4096;

const FONT_SET: [u16; 80] = [
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

#[derive(Debug)]
struct Chip8Engine {
    // delay_timer: u8,
    // sound_timer: u8,
    // keyboard: u16,
    // stack: Vec<u16>,
    memory: [u8; MEMORY_SIZE],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,

    // TODO(ben): this is a simple approach but can be reduced from
    // 2kb -> 256bytes with bit-packing. Maybe refactor.
    display: [u8; DISPLAY_SIZE],
}

#[derive(Debug)]
struct Opcode {
    x: u8,
    y: u8,
    n: u8,
    kk: u8,
    addr: u16,
}

impl Chip8Engine {
    pub fn new(rom_file: &str) -> Result<Self, Box<std::io::Error>> {
        let rom = std::fs::read(rom_file)?;

        let mut memory = [0; MEMORY_SIZE];
        // initialize font_set
        for (i, &font_byte) in FONT_SET.iter().enumerate() {
            memory[i + 0x50] = font_byte as u8;
        }

        // read rom into memory
        for (i, byte) in rom.iter().enumerate() {
            memory[0x200 + i] = *byte;
        }

        Ok(Chip8Engine {
            program_counter: 0x200,
            memory,
            registers: [0; 16],
            index_register: 0,
            display: [0; DISPLAY_SIZE],
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
                self.memory[self.program_counter as usize],
                self.memory[self.program_counter as usize + 1],
            ];

            self.program_counter += 2;

            u16::from_be_bytes(slice)
        };

        let opcode = Opcode {
            x: ((code & 0x0F00) >> 8) as u8,
            y: ((code & 0x00F0) >> 4) as u8,
            n: (code & 0x000F) as u8,
            kk: (code & 0x00FF) as u8,
            addr: (code & 0x0FFF) as u16,
        };

        match code & 0xF000 {
            0x0000 => match code & 0x00F0 {
                // 0x00E0 - CLS
                0x00E0 => {
                    for pixel in self.display.iter_mut() {
                        *pixel = 0x0;
                    }
                }
                _ => {
                    todo!()
                }
            },

            // 0x1nnn - JP addr
            0x1000 => {
                self.program_counter = opcode.addr;
            }

            // 0x6xkk - LD Vx, byte
            0x6000 => self.registers[opcode.x as usize] = opcode.kk,

            // 0x7xkk - ADD Vx, byte
            0x7000 => self.registers[opcode.x as usize] += opcode.kk,

            // 0xAnnn - LD I, addr
            0xA000 => {
                self.index_register = opcode.addr;
            }

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
                let x = (self.registers[opcode.x as usize] % DISPLAY_WIDTH as u8) as usize;
                let y = (self.registers[opcode.y as usize] % DISPLAY_HEIGHT as u8) as usize;

                self.registers[0xF] = 0;

                let start = self.index_register as usize;

                for i in 0..opcode.n as usize {
                    let raw_byte = self.memory[start + i];

                    for j in 0..8 {
                        let curr_pixel = (raw_byte << j & 0x80) >> 7;
                        let cx = x + j;
                        let cy = y + i;
                        let index = cx + cy * DISPLAY_WIDTH;

                        let display_pixel = self.display.get(index).unwrap_or(&0);

                        // Collision detection.
                        if *display_pixel == 1 && curr_pixel == 1 {
                            self.registers[0xF] = 1;
                        }

                        if *display_pixel == 0 && curr_pixel == 1 {
                            if let Some(v) = self.display.get_mut(index) {
                                *v = 1;
                            }
                        }
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

    for _ in 0..2000 {
        chip_8.tick()?;
    }

    // draw
    for y in 0..32 {
        for x in 0..64 {
            let index = x + y * 64;
            let pixel = if chip_8.display[index] == 1 { "*" } else { " " };
            print!("{}", pixel);
        }
        println!();
    }

    Ok(())
}
