use rand::Rng;
use std::error::Error;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const MEMORY_SIZE: usize = 4096;

const FONT_SET: [u8; 80] = [
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
pub struct Chip8Engine {
    // delay_timer: u8,
    // sound_timer: u8,
    // keyboard: u16,
    stack: Vec<u16>,
    stack_pointer: u8,
    memory: [u8; MEMORY_SIZE],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,

    // TODO(ben): this is a simple approach but can be reduced from
    // 2kb -> 256bytes with bit-packing. Maybe refactor.
    pub display: [u8; DISPLAY_SIZE], // idk if I want this to be public or not, yet...
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
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        for (i, &font_byte) in FONT_SET.iter().enumerate() {
            memory[0x50 + i] = font_byte;
        }

        Chip8Engine {
            stack: Vec::new(),
            stack_pointer: 0,
            program_counter: 0x200,
            memory,
            registers: [0; 16],
            index_register: 0,
            display: [0; DISPLAY_SIZE],
        }
    }

    /// loads a ROM into the CHIP-8 Engine.
    /// for now, we don't do any sanity checks here. I don't know what sanity checks we could do to
    /// ensure that what you provided is something that the Engine can actually run.
    pub fn load(&mut self, source: &[u8]) {
        for (i, byte) in source.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    // Ideally, I want this to be the main operation. But this might be harder to hook into when
    // running in an event loop. Also, what is "run"? Does that include configuring all of the
    // keyboard inputs, sounds, display?
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
    // This is public for now, due to development. We'll see if I want to make this private down
    // the road.
    pub fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        let mut rng = rand::thread_rng();
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
                // 0x00EE - RET
                0x00EE => {
                    self.program_counter = *self.stack.last().unwrap_or(&0);
                    self.stack_pointer -= 1;
                }
                _ => unimplemented!("unknown instruction"),
            },

            // 0x1nnn - JP addr
            0x1000 => self.program_counter = opcode.addr,

            // 0x2nnn - CALL addr
            0x2000 => {
                self.stack_pointer += 1;
                self.stack.push(self.program_counter);
                self.program_counter = opcode.addr;
            }

            // 0x3xkk - SE Vx, byte
            0x3000 => {
                if self.registers[opcode.x as usize] == opcode.kk {
                    self.program_counter += 2;
                }
            }

            // 0x4xkk - SNE Vx, byte
            0x4000 => {
                if self.registers[opcode.x as usize] != opcode.kk {
                    self.program_counter += 2;
                }
            }

            // 0x5xy0 - SE Vx, Vy
            0x5000 => {
                if self.registers[opcode.x as usize] == self.registers[opcode.y as usize] {
                    self.program_counter += 2;
                }
            }

            // 0x6xkk - LD Vx, byte
            0x6000 => self.registers[opcode.x as usize] = opcode.kk,

            // 0x7xkk - ADD Vx, byte
            0x7000 => self.registers[opcode.x as usize] += opcode.kk,

            0x8000 => match code & 0xF00F {
                // 8xy0 - LD Vx, Vy
                0x8000 => todo!(),
                // 8xy1 - OR Vx, Vy
                0x8001 => todo!(),
                // 8xy2 - AND Vx, Vy
                0x8002 => todo!(),
                // 8xy3 - XOR Vx, Vy
                0x8003 => todo!(),
                // 8xy4 - ADD Vx, Vy
                0x8004 => todo!(),
                // 8xy5 - SUB Vx, Vy
                0x8005 => todo!(),
                // 8xy6 - SHR Vx {, Vy}
                0x8006 => todo!(),
                // 8xy7 - SUBN Vx, Vy
                0x8007 => todo!(),
                // 8xyE - SHL Vx {, Vy}
                0x800E => todo!(),
                _ => unimplemented!("unknown instruction"),
            },

            // 0x9xy0 - SNE Vx, Vy
            0x9000 => {
                if self.registers[opcode.x as usize] != self.registers[opcode.y as usize] {
                    self.program_counter += 2;
                }
            }

            // 0xAnnn - LD I, addr
            0xA000 => {
                self.index_register = opcode.addr;
            }

            // Bnnn - JP V0, addr
            //
            // **AMBIGIOUS INSTRUCTION**.
            //
            // COSMAC VIP reads 0xBNNN
            // CHIP-48 and SUPER-CHIP reads 0xBXNN
            // Later, I'll add a flag to control this quirk.
            0xB000 => {
                self.program_counter = opcode.addr + self.registers[0] as u16;
            }

            // Cxkk - RND Vx, byte
            0xC000 => {
                let random_num: u8 = rng.gen_range(0..=255);
                self.registers[opcode.x as usize] = random_num & opcode.kk;
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
            0xE000 => match code & 0xF0FF {
                // Ex9E - SKP Vx
                0xE09E => todo!(),
                // ExA1 - SKNP Vx
                0xE0A1 => todo!(),
                _ => unimplemented!("unknown instruction"),
            },
            0xF000 => match code & 0xF0FF {
                // Fx07 - LD Vx, DT
                0xF007 => todo!(),
                // Fx0A - LD Vx, K
                0xF00A => todo!(),
                // Fx15 - LD DT, Vx
                0xF015 => todo!(),
                // Fx18 - LD ST, Vx
                0xF018 => todo!(),
                // Fx1E - ADD I, Vx
                0xF01E => todo!(),
                // Fx29 - LD F, Vx
                0xF029 => todo!(),
                // Fx33 - LD B, Vx
                0xF033 => todo!(),
                // Fx55 - LD [I], Vx
                0xF055 => todo!(),
                // Fx65 - LD Vx, [I]
                0xF065 => todo!(),
                _ => unimplemented!(),
            },
            _ => unimplemented!("unknown instruction"),
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Chip8Engine, FONT_SET};

    #[test]
    fn new() {
        let engine = Chip8Engine::new();

        assert_eq!(engine.memory[0x50..0xA0], FONT_SET);
        assert_eq!(engine.program_counter, 0x200);
    }

    #[test]
    fn load() {
        let mut engine = Chip8Engine::new();
        let source = [0x00, 0xE0, 0xD3, 0x51];
        engine.load(&source);

        assert_eq!(engine.memory[0x200..0x204], source);
    }

    #[test]
    fn tick() {
        let mut engine = Chip8Engine::new();
        let source = [0x00, 0xE0, 0xD3, 0x51];
        engine.load(&source);

        let _ = engine.tick();
        assert_eq!(engine.program_counter, 0x202);

        let _ = engine.tick();
        assert_eq!(engine.program_counter, 0x204);
    }
}
