use std::fs;
use std::env::args;
use std::io::Read;

const MEMSIZE_6502: u16 = 0xFFFF;

// 7  bit  0
// ---- ----
// NV1B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ (No CPU effect; see: the B flag)
// ||+------- (No CPU effect; always pushed as 1)
// |+-------- Overflow
// +--------- Negative

pub const STATUS_FLAG_N: u8 = 0x01 << 7;
pub const STATUS_FLAG_V: u8 = 0x01 << 6;
pub const STATUS_FLAG_1: u8 = 0x01 << 5;
pub const STATUS_FLAG_B: u8 = 0x01 << 4;
pub const STATUS_FLAG_D: u8 = 0x01 << 3;
pub const STATUS_FLAG_I: u8 = 0x01 << 2;
pub const STATUS_FLAG_Z: u8 = 0x01 << 1;
pub const STATUS_FLAG_C: u8 = 0x01;

enum  AddressingMode {
    AddressingAbsolute,
    AddressingAbsoluteX,
    AddressingAbsoluteY,
    AddressingImmediate,
    AddressingImplied,
    AddressingIndirectX,
    AddressingIndirectY,
    AddressingZeroPage,
    AddressingZeroPageX,
    AddressingZeroPageY,
}

struct	Ins6502 {
    opcode: u8,
    mnem: String,
    addressing_mode: AddressingMode,
}

struct Regs6502 {
    pc:	u16,	// program counter
    sp:	u8,	// stack pointer
    p:	u8,	// status register
    a:	u8,	// accumulator
    x:	u8,	// x reg
    y:	u8,	// y reg
}

struct	Bus6502 {
    vram: [u8; MEMSIZE_6502 as usize],
}

pub struct	Cpu6502 {
    regs:	Regs6502,
    bus:	Bus6502,
    ins:	Vec<Ins6502>,
}

fn status_reset_flag(regs: &mut Regs6502, to_clear: char){
    match to_clear {
	'p'  => { regs.p = 0; }
	'a'  => { regs.a = 0; }
	'x'  => { regs.x = 0; }
	'y'  => { regs.y = 0; }
	_ => unimplemented!()
    }
}

// not implementing implied addressing because it's implied
impl	Cpu6502 {
    pub fn	new() -> Self {
	Cpu6502 {
	    regs: { Regs6502 {
		pc: 0,
		sp: 0,
		p:  0,
		a:  0,
		x:  0,
		y:  0
	    }},
	    bus: { Bus6502 {
		vram: [0; MEMSIZE_6502 as usize]
	    }},
	    ins: vec![
		Ins6502 {opcode: 0x00, mnem: "BRK".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x18, mnem: "CLC".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x18, mnem: "CLD".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x01, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingIndirectX},
		Ins6502 {opcode: 0xA9, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingImmediate},
		Ins6502 {opcode: 0xAA, mnem: "TAX".to_string(), addressing_mode: AddressingMode::AddressingImplied},
	    ]
	}
    }

    fn set_status_bit (self: &mut Self, reg_id: char, bit: char) {
	let mut target_register: &u8 = &0;

	match reg_id {
	    'a' => {target_register = &self.regs.a;}
	    'x' => {target_register = &self.regs.x;}
	    'y' => {target_register = &self.regs.y;}
	    _ => unimplemented!()
	}
	match bit {
	    'Z' => {
		if *target_register == 0 {
		    self.regs.p = self.regs.p | STATUS_FLAG_Z;
		}
		else {
		    self.regs.p = self.regs.p & !STATUS_FLAG_Z;
		}
	    }
	    'N' => {
		if (*target_register & STATUS_FLAG_N) == STATUS_FLAG_N {
		    self.regs.p = self.regs.p | STATUS_FLAG_N;
		}
		else {
		    self.regs.p = self.regs.p & !STATUS_FLAG_N;
		}
	    }
	    _ => todo!()
	}
    }


    fn get_operand(self: &mut Self, instruction_index: u8, instruction_buffer: &Vec<u8>) -> u16 {
	let mode = &self.ins[instruction_index as usize].addressing_mode;

	match mode {
	    AddressingMode::AddressingImplied	=> { return 0; } 

	    AddressingMode::AddressingImmediate	=> {
		let immediate: u16 = instruction_buffer[self.regs.pc as usize] as u16;
		self.regs.pc += 1;
		immediate
	    }

	    AddressingMode::AddressingAbsolute	=> {
		let absolute: u16 = (instruction_buffer[(self.regs.pc + 1) as usize] << 4) as u16
		    | (instruction_buffer[(self.regs.pc + 2) as usize] as u16);
		self.regs.pc += 2;
		absolute
	    }

	    AddressingMode::AddressingZeroPage	=> {
		let zp: u16 = instruction_buffer[self.regs.pc as usize] as u16;
		let val: u16 = self.bus.vram[zp as usize] as u16;
		self.regs.pc += 2;
		val
	    }

	    _ => todo!()
	}
    }

    fn instruction_fetch(self: &mut Self, rom_bytes: &Vec<u8>) -> u8 {
	let instruction = rom_bytes[self.regs.pc as usize];
	self.regs.pc = self.regs.pc + 1;
	instruction
    }

    fn instruction_execute(self: &mut Self, opcode: u8, instruction_buffer: &mut Vec<u8>) -> i32{

	// https://www.masswerk.at/6502/6502_instruction_set.html
	let index_of_ins_in_vec = self.ins.iter().position(|ins| ins.opcode == opcode).unwrap();
	let operand = self.get_operand(index_of_ins_in_vec as u8, instruction_buffer);
	let ins: &Ins6502 = &self.ins[index_of_ins_in_vec];

	println!("{}, {:#x}", ins.mnem, operand);

	match &ins.mnem as &str {
	    "BRK" => { return -1; }
	    "ORA" => {
		self.regs.a = self.regs.a | operand as u8;
		self.set_status_bit('a', 'N');
		self.set_status_bit('a', 'Z');
	    }

	    "CLC" => { // clc
		status_reset_flag(&mut self.regs, 'C');
	    }

	    "LDA" => {
		self.regs.a = operand as u8;
		self.set_status_bit('a', 'N');
		self.set_status_bit('a', 'Z');
	    }

	    "TAX" => {
		self.regs.x = self.regs.a;
	    }

	    _ => todo!()
	}
	return 0;
    }

    fn run(self: &mut Self, rom_buff: &mut Vec<u8>){
	println!("Executing...\n");
	loop {
	    let op = self.instruction_fetch(&rom_buff);
	    let status = self.instruction_execute(op, rom_buff);
	    if status == -1 {
		return;
	    }
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

    let rom_filename;
    let mut rom_buff;

    if args.len() > 1 {
	rom_filename = &args[1];
	println!("Reading contents of {}...", rom_filename);
	rom_buff = file_to_u8_vector(rom_filename);
    }
    else {
	rom_buff = vec!(0xa9, 0x05, 0x00);
    }

    let mut cpu = Cpu6502::new();

    println!("Entering CPU loop!");
    cpu.run(&mut rom_buff);
    println!("Done! See ya.");
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec!(0xa9, 0x05, 0x00);

	cpu.run(&mut rom_buff);
	println!("{:b}", cpu.regs.p);
	assert_eq!(cpu.regs.a, 0x05);
	assert!(cpu.regs.p & STATUS_FLAG_Z == 0b00);
	assert!(cpu.regs.p & STATUS_FLAG_N == 0b00);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa9, 0x00, 0x00];

	cpu.run(&mut rom_buff);
	assert!(cpu.regs.p & STATUS_FLAG_Z == 0b10);
    }

}
