use std::error::Error;
use std::ops::{Deref, DerefMut};

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "./src/IBM Logo.ch8";
    let rom = std::fs::read(file_name).unwrap();
    let mut chip_8 = Chip8Engine::new(&rom); // TODO(ben): does having the instance itself be
                                             // mutable make sense?

    chip_8.run()?;
    println!("{:?}", chip_8.display);

    Ok(())
}

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

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
    screen: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT], // TODO(ben): use constants for these values.
    width: usize,
    height: usize,
}
impl Display {
    fn new() -> Self {
        Display {
            screen: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
        }
    }

    // [
    //     0,0,0,0,0,
    //     0,0,0,0,0,
    //     0,0,0,0,0,
    //     0,0,0,0,0,
    //     0,0,0,0,0
    // ]

    // TODO(ben): where should modulo operation be? In the callsite or in the function?
    /// write_pixel writes a pixel value at the point x, y, returning the new value of that pixel.
    /// If that point does not exist in the display, it will return none.
    fn write_pixel(&mut self, x: usize, y: usize, value: bool) -> Option<bool> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.screen[x + y * self.height] = value;
            self.get_pixel(x, y)
        }
    }

    /// get_pixel returns a pixel value at the point x, y. If that point does not exist in the
    /// display, it will return none.
    fn get_pixel(&self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.screen[x + y * self.height])
        }
    }
}

impl Deref for Display {
    type Target = [bool; DISPLAY_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.screen
    }
}

impl DerefMut for Display {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.screen
    }
}

enum Instructions {
    Clear,
    Jump,
    SetVX,
    AddVX,
    SetI,
    Draw,
}

struct Opcode {
    x: u8, //TODO(Ben): this is exploration. Is usize better or u16? u8? boolean?
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
    instruction: Instructions,
}

impl Chip8Engine {
    pub fn new(rom: &Vec<u8>) -> Self {
        // initialize font_set
        let mut memory = vec![0; 2096];
        for (i, &font_byte) in FONT_SET.iter().enumerate() {
            memory[i + 0x50] = font_byte as u8;
        }

        for (i, byte) in rom.iter().enumerate() {
            memory[0x200 + i] = *byte;
        }

        Chip8Engine {
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
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        for _ in 0..240 {
            self.tick()?;
        }

        Ok(())
    }

    /// tick executes one operation at a time.
    fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        let code = self.fetch();
        println!("CODE: {:#02x}", code);
        let instruction = self.decode(code).ok_or_else(|| {
            println!("OUTPUT: {:?}", self.display.screen);

            "unknown instruction"
        })?;
        self.execute(instruction)?;

        Ok(())
    }

    /// fetch grabs the next two instructions, returning a u16.
    fn fetch(&mut self) -> u16 {
        let slice = [
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ];

        self.program_counter += 2;

        u16::from_be_bytes(slice)
    }

    fn decode(&self, code: u16) -> Option<Opcode> {
        // TODO(ben): can decoding fail? Likely, yes. Should
        // error with Result type.
        match code & 0xF000 {
            0x0000 => match code & 0x00F0 {
                0x00E0 => Some(create_opcode(code, Instructions::Clear)),
                _ => None,
            },
            0x1000 => Some(create_opcode(code, Instructions::Jump)),
            0x6000 => Some(create_opcode(code, Instructions::SetVX)),
            0x7000 => Some(create_opcode(code, Instructions::AddVX)),
            0xA000 => Some(create_opcode(code, Instructions::SetI)),
            0xD000 => Some(create_opcode(code, Instructions::Draw)),
            _ => None,
        }
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), Box<dyn Error>> {
        match opcode.instruction {
            Instructions::Clear => {
                for pixel in self.display.iter_mut() {
                    *pixel = false;
                }
            }
            Instructions::Jump => self.program_counter = opcode.nnn.into(),
            // TODO(ben): why cast to usize? any alternatives?
            Instructions::SetVX => self.registers[opcode.x as usize] = opcode.nn,
            Instructions::AddVX => self.registers[opcode.x as usize] += opcode.nn,
            Instructions::SetI => self.index_register = opcode.nnn,
            Instructions::Draw => {
                let x = self.registers[opcode.x as usize] as usize % self.display.width;
                let y = self.registers[opcode.y as usize] as usize % self.display.height;
                self.registers[0xF] = 0; // flag register
                                         //
                                         // TODO(ben): like what the fuck. Seriously? So much casting! Turn it all into
                                         // usizes. Maybe.
                let input = &self.memory[(self.index_register as usize)
                    ..((self.index_register + opcode.n as u16) as usize)];

                // TODO(ben): maybe use bitvec? more clever way to batch/XOR/Check flag for 8 bytes at a time
                // also... yeah this is messy.
                //
                // TODO(ben): get modulo operation down.
                // TODO(ben): ensure clipping rules are valid. Do not overflow your array.
                for i in input.iter() {
                    for j in (0..8).rev() {
                        let bit = (i >> j) & 1;

                        let offset: u8 = (i * 8 + j).into();
                        let x_pos = x + offset as usize;
                        let y_pos = y;

                        match self.display.get_pixel(x_pos.into(), y_pos.into()) {
                            Some(curr_pixel) => {
                                let xor = bit ^ curr_pixel as u8;
                                match self
                                    .display
                                    .write_pixel(x_pos.into(), y_pos.into(), xor != 0)
                                {
                                    Some(new_pixel) => {
                                        if new_pixel {
                                            self.registers[0xF] = 1;
                                        }
                                    }
                                    None => continue,
                                }
                            }
                            None => continue,
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// TODO(ben): should this be within the Chip-8 Engine struct?
fn create_opcode(code: u16, instruction: Instructions) -> Opcode {
    Opcode {
        x: (code & 0x0100 >> 2) as u8,
        y: (code & 0x0011 >> 1) as u8,
        n: (code & 0x0001) as u8,
        nn: (code & 0x0011) as u8,
        nnn: (code & 0x0111) as u16,
        instruction,
    }
}

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
