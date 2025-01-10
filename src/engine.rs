use rand::Rng;
use std::error::Error;

pub const DISPLAY_WIDTH: usize = 0x40;
pub const DISPLAY_HEIGHT: usize = 0x20;
pub const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const MEMORY_SIZE: usize = 0x1000;
const FONT_START: usize = 0x50;
const VF_REGISTER: usize = 0xF;

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
    pub display: [u8; DISPLAY_SIZE],
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
    /// Creates a new CHIP-8 Engine.
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        for (i, &font_byte) in FONT_SET.iter().enumerate() {
            memory[FONT_START + i] = font_byte;
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

    /// Loads a ROM into the CHIP-8 Engine.
    pub fn load(&mut self, source: &[u8]) {
        for (i, byte) in source.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    /// Runs the fetch/decode/execute loop for a single 16 bit instruction.
    pub fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        println!("pc: {}", self.program_counter);
        let mut rng = rand::thread_rng();
        let code = {
            let slice = [
                self.memory[self.program_counter as usize],
                self.memory[self.program_counter as usize + 1],
            ];

            self.program_counter += 2;

            u16::from_be_bytes(slice)
        };
        println!("code: {:#02x}", code);

        let opcode = Opcode {
            x: ((code & 0x0F00) >> 8) as u8,
            y: ((code & 0x00F0) >> 4) as u8,
            n: (code & 0x000F) as u8,
            kk: (code & 0x00FF) as u8,
            addr: (code & 0x0FFF) as u16,
        };

        match code & 0xF000 {
            0x0000 => match code & 0x00FF {
                // 0x00E0 - CLS
                0x00E0 => {
                    for pixel in self.display.iter_mut() {
                        *pixel = 0x0;
                    }
                }
                // 0x00EE - RET
                0x00EE => {
                    println!("ooee");
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
            0x7000 => {
                let (sum, _) = self.registers[opcode.x as usize].overflowing_add(opcode.kk);
                self.registers[opcode.x as usize] = sum;
            }

            0x8000 => match code & 0xF00F {
                // 8xy0 - LD Vx, Vy
                0x8000 => self.registers[opcode.x as usize] = self.registers[opcode.y as usize],
                // 8xy1 - OR Vx, Vy
                0x8001 => self.registers[opcode.x as usize] |= self.registers[opcode.y as usize],
                // 8xy2 - AND Vx, Vy
                0x8002 => self.registers[opcode.x as usize] &= self.registers[opcode.y as usize],
                // 8xy3 - XOR Vx, Vy
                0x8003 => self.registers[opcode.x as usize] ^= self.registers[opcode.y as usize],
                // 8xy4 - ADD Vx, Vy
                0x8004 => {
                    self.registers[VF_REGISTER] = 0;
                    let vx = self.registers[opcode.x as usize];
                    let vy = self.registers[opcode.y as usize];

                    let (sum, overflow) = vx.overflowing_add(vy);

                    // Overflow detection.
                    self.registers[VF_REGISTER] = overflow as u8;
                    self.registers[opcode.x as usize] = sum;
                }
                // 8xy5 - SUB Vx, Vy
                0x8005 => {
                    self.registers[VF_REGISTER] = 1;
                    let vx = self.registers[opcode.x as usize];
                    let vy = self.registers[opcode.y as usize];
                    let (res, underflow) = vx.overflowing_sub(vy);

                    // Underflow detection.
                    self.registers[VF_REGISTER] = underflow as u8;
                    self.registers[opcode.x as usize] = res;
                }
                // 8xy6 - SHR Vx {, Vy}
                //
                // **AMBIGIOUS INSTRUCTION**.
                // COSMAC VIP: set Vx = Vy then shift Vx >> 1. Vf = shifted value.
                // CHIP-48 / SuPER-CHIP: Shift Vx >> 1. Ignore Vy. Vf = shifted value.
                0x8006 => {
                    self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
                    self.registers[VF_REGISTER] = self.registers[opcode.x as usize] & 1;
                    self.registers[opcode.x as usize] >>= 1;
                }
                // 8xy7 - SUBN Vx, Vy
                0x8007 => {
                    self.registers[VF_REGISTER] = 1;
                    let vx = self.registers[opcode.x as usize];
                    let vy = self.registers[opcode.y as usize];
                    let (res, underflow) = vy.overflowing_sub(vx);

                    // Underflow detection.
                    self.registers[VF_REGISTER] = underflow as u8;
                    self.registers[opcode.x as usize] = res;
                }
                // 8xyE - SHL Vx {, Vy}
                //
                // **AMBIGIOUS INSTRUCTION**.
                //
                // COSMAC VIP: set Vx = Vy then shift Vx >> 1.
                // CHIP-48 / SuPER-CHIP: Shift Vx >> 1. Ignore Vy. Vf = shifted value.
                0x800E => {
                    self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
                    self.registers[VF_REGISTER] = self.registers[opcode.x as usize] & 0x80;
                    self.registers[opcode.x as usize] <<= 1;
                }
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
                println!("post A000");
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

                self.registers[VF_REGISTER] = 0;

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
                            self.registers[VF_REGISTER] = 1;
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
                0xF007 => todo!(), // TIMER
                // Fx0A - LD Vx, K
                //
                // On the original COSMAC VIP, the key was only registered when it was pressed and then released.
                0xF00A => todo!(), // GET KEY
                // Fx15 - LD DT, Vx
                0xF015 => todo!(), // TIMER
                // Fx18 - LD ST, Vx
                0xF018 => todo!(), // TIMER
                // Fx1E - ADD I, Vx
                //
                // SEMI-AMBIGUOUS INSTRUCTION: Amiga interpreter "overflows" from 0xFFF to0x1000.
                // Maybe check for this. If you care.
                0xF01E => {
                    self.index_register += self.registers[opcode.x as usize] as u16;
                }
                // Fx29 - LD F, Vx
                0xF029 => {
                    let font_length = 5;
                    self.index_register =
                        self.memory[FONT_START + font_length * opcode.x as usize] as u16;
                }
                // Fx33 - LD B, Vx
                0xF033 => {
                    let vx = self.registers[opcode.x as usize];

                    let ones = vx % 10;
                    let tens = (vx / 10) % 10;
                    let hundreds = (vx / 100) % 10;

                    let index = self.index_register as usize;

                    self.memory[index] = hundreds;
                    self.memory[index + 1] = tens;
                    self.memory[index + 2] = ones;
                }
                // Fx55 - LD [I], Vx
                //
                // **AMBIGIOUS INSTRUCTION**.
                0xF055 => {
                    let index = self.index_register as usize;
                    for i in 0..=opcode.x {
                        self.memory[index + i as usize] = self.registers[i as usize];
                    }
                }
                // Fx65 - LD Vx, [I]
                //
                // **AMBIGIOUS INSTRUCTION**.
                0xF065 => {
                    let index = self.index_register as usize;
                    for i in 0..=opcode.x {
                        self.registers[i as usize] = self.memory[index + i as usize];
                    }
                }
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
