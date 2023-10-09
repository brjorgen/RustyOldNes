use std::fs;
use std::env::args;
use std::io::Read;

struct Regs6502 {
    pc:	u16,
    sp:	u8,
    p:	u8,
    a:	u8,
    x:	u8,
    y:	u8,
}

struct	Bus6502 {
    vram: [u8; 4096],
}

struct	Cpu6502 {
    regs:	Regs6502,
    bus:	Bus6502,
}

struct  InstructionParsed {
    hi: u8,
    lo: u8,
}

fn	addressing_immediate(instruction_buffer: Vec<u8>, pc: &mut u16) -> u8 {
    let immediate: u8 = instruction_buffer[(*pc + 1) as usize];
    *pc += 1;
    immediate
}

fn	addressing_absolute(instruction_buffer: Vec<u8>, pc: &mut u16) -> u16 {
    let absolute: u16 = (instruction_buffer[(*pc + 1) as usize] << 8) as u16
	| (instruction_buffer[(*pc + 2) as usize] as u16);
    *pc += 2;
    absolute
}

fn	addressing_zero_page(vram: [u8; 4096], instruction_buffer: Vec<u8>, pc: &mut u16) -> u16 {
    let zp: u16 = addressing_immediate(instruction_buffer, pc) as u16;
    let val: u16 = (0x00 << 8) as u16 | vram[zp as usize] as u16;
    *pc += 2;
    val
}

// not implementing implied addressing because it's implied

impl	Cpu6502 {
    pub fn	new() -> Self {
	 Cpu6502 {
	     regs: { Regs6502 {
		     pc: 0x100,
		     sp: 0,
		     p:  0,
		     a:  0,
		     x:  0,
		     y:  0
	     }},
	     bus: { Bus6502 {
		 vram: [0; 4096]
	     }}
	}
    }

    fn instruction_fetch(self: &Self, rom_bytes: &Vec<u8>) -> u8 {
	let instruction = rom_bytes[self.regs.pc as usize];
	instruction
    }

    fn instruction_execute(self: &mut Self, opcode: u8, instruction_buffer: &mut Vec<u8>){
	// https://www.masswerk.at/6502/6502_instruction_set.html
	match opcode {
	    0x00 => { return; }
	    // 0x01 => {   }
	    _ => todo!()
	}
    }
}

fn	file_to_u8_vector(filename: &String) -> Vec<u8> {
    let mut f = fs::File::open(&filename).expect("File not found.");
    let metadata = fs::metadata(&filename).expect("Unable to read the file's metadata.");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("Buffer overflow.");

    buffer
}

fn	main() {
    println!("Welcome to RustyOldNes - A NES emulator, written in Rust.");
    let args: Vec<String> = args().collect();
    let rom_filename: &String = &args[1];

    println!("Reading contents of {}...", rom_filename);
    let mut rom_buff: Vec<u8> = file_to_u8_vector(rom_filename);
    let mut cpu = Cpu6502::new();

    loop {
	let op = cpu.instruction_fetch(&rom_buff);
	cpu.instruction_execute(op, &mut rom_buff);
    }
}
