fn main() {
    let file_name = "IBM Logo.ch8";
    let rom = std::fs::read(file_name).unwrap();
    let chip_8 = Chip8Engine::new(&rom);

    println!("{:?}", chip_8);
}

#[derive(Debug)]
struct Chip8Engine<'a> {
    rom: &'a [u8],
    memory: [u8; 2096],
    registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    delay_timer: u8,
    sound_timer: u8,
    display: [bool; 64 * 32],
    keyboard: u16,
    stack: Vec<u16>,
}

impl<'a> Chip8Engine<'a> {
    pub fn new(rom: &'a [u8]) -> Self {
        // initialize font_set
        let mut memory = [0; 2096];
        for (i, &byte) in FONT_SET.iter().enumerate() {
            memory[i] = byte as u8;
        }

        Chip8Engine {
            rom,
            program_counter: 200,
            memory,
            registers: [0; 16],
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keyboard: 0,
            stack: Vec::new(),
        }
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
