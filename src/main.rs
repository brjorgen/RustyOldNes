use std::fs;
use std::env::args;
use std::io::Read;
use std::fmt;

const MEMSIZE_6502: u16 = 0xFFFF;
const MEMSTART_CARTRIDGE_ROM_6502: u16 = 0x8000;
const MEMSTART_STACK_6502: u16 = 0x1FF;
const MEMSTART_STACK_SIZE: u16 = 0xFF;

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum  AddressingMode {
    AddressingAbsolute,
    AddressingAbsoluteX,
    AddressingAbsoluteY,
    AddressingImmediate,
    AddressingImplied,
    AddressingIndirect,
    AddressingIndirectX,
    AddressingIndirectY,
    AddressingZeroPage,
    AddressingZeroPageX,
    AddressingZeroPageY,
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
	    AddressingMode::AddressingAbsolute => write!(f, "AddressingAbsolute"),
	    AddressingMode::AddressingAbsoluteX => write!(f, "AddressingAbsoluteX"),
	    AddressingMode::AddressingAbsoluteY => write!(f, "AddressingAbsoluteY"),
	    AddressingMode::AddressingImmediate => write!(f, "AddressingImmediate"),
	    AddressingMode::AddressingImplied => write!(f, "AddressingImplied"),
	    AddressingMode::AddressingIndirect => write!(f, "AddressingIndirect"),
	    AddressingMode::AddressingIndirectX => write!(f, "AddressingIndirectX"),
	    AddressingMode::AddressingIndirectY => write!(f, "AddressingIndirectY"),
	    AddressingMode::AddressingZeroPage => write!(f, "AddressingZeroPage"),
	    AddressingMode::AddressingZeroPageX => write!(f, "AddressingZeroPageX"),
	    AddressingMode::AddressingZeroPageY => write!(f, "AddressingZeroPageY")
        }
    }
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

// not implementing implied addressing because it's implied
impl	Cpu6502 {
    pub fn	new() -> Self {
	Cpu6502 {
	    regs: { Regs6502 {
		pc: MEMSTART_CARTRIDGE_ROM_6502,
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
		Ins6502 {opcode: 0xEA, mnem: "NOP".to_string(), addressing_mode: AddressingMode::AddressingImplied},

		Ins6502 {opcode: 0x18, mnem: "CLC".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0xD8, mnem: "CLD".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x58, mnem: "CLI".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0xB8, mnem: "CLI".to_string(), addressing_mode: AddressingMode::AddressingImplied},

		Ins6502 {opcode: 0x09, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingImmediate},
		Ins6502 {opcode: 0x05, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x15, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0x0D, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0x1D, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},
		Ins6502 {opcode: 0x19, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteY},
		Ins6502 {opcode: 0x01, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingIndirectX},
		Ins6502 {opcode: 0x11, mnem: "ORA".to_string(), addressing_mode: AddressingMode::AddressingIndirectY},

		Ins6502 {opcode: 0xA9, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingImmediate},
		Ins6502 {opcode: 0xA5, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0xB5, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0xAD, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0xBD, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},
		Ins6502 {opcode: 0xB9, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteY},
		Ins6502 {opcode: 0xA1, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingIndirectX},
		Ins6502 {opcode: 0xA1, mnem: "LDA".to_string(), addressing_mode: AddressingMode::AddressingIndirectY},

		Ins6502 {opcode: 0xA2, mnem: "LDX".to_string(), addressing_mode: AddressingMode::AddressingImmediate},
		Ins6502 {opcode: 0xA0, mnem: "LDY".to_string(), addressing_mode: AddressingMode::AddressingImmediate},

		Ins6502 {opcode: 0x6C, mnem: "JMP".to_string(), addressing_mode: AddressingMode::AddressingIndirect},
		Ins6502 {opcode: 0x4C, mnem: "JMP".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},

		Ins6502 {opcode: 0x84, mnem: "STY".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x94, mnem: "STY".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0x8C, mnem: "STY".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},

		Ins6502 {opcode: 0x85, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x95, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0x8D, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0x9D, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},
		Ins6502 {opcode: 0x99, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteY},
		Ins6502 {opcode: 0x81, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingIndirectX},
		Ins6502 {opcode: 0x91, mnem: "STA".to_string(), addressing_mode: AddressingMode::AddressingIndirectY},

		Ins6502 {opcode: 0x86, mnem: "STX".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x96, mnem: "STX".to_string(), addressing_mode: AddressingMode::AddressingZeroPageY},
		Ins6502 {opcode: 0x8E, mnem: "STX".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},


		Ins6502 {opcode: 0xAA, mnem: "TAX".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0xA8, mnem: "TAY".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0xA8, mnem: "TSX".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x8A, mnem: "TXA".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x9A, mnem: "TXS".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x98, mnem: "TYA".to_string(), addressing_mode: AddressingMode::AddressingImplied},


		Ins6502 {opcode: 0xE6, mnem: "INC".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0xF6, mnem: "INC".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0xEE, mnem: "INC".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0xFE, mnem: "INC".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},
		Ins6502 {opcode: 0xE8, mnem: "INX".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0xC8, mnem: "INY".to_string(), addressing_mode: AddressingMode::AddressingImplied},

		Ins6502 {opcode: 0x48, mnem: "PHA".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x08, mnem: "PHP".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x68, mnem: "PLA".to_string(), addressing_mode: AddressingMode::AddressingImplied},
		Ins6502 {opcode: 0x28, mnem: "PLP".to_string(), addressing_mode: AddressingMode::AddressingImplied},

		Ins6502 {opcode: 0x26, mnem: "ROL".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x36, mnem: "ROL".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0x2E, mnem: "ROL".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0x3E, mnem: "ROL".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},

		Ins6502 {opcode: 0x66, mnem: "ROR".to_string(), addressing_mode: AddressingMode::AddressingZeroPage},
		Ins6502 {opcode: 0x76, mnem: "ROR".to_string(), addressing_mode: AddressingMode::AddressingZeroPageX},
		Ins6502 {opcode: 0x6E, mnem: "ROR".to_string(), addressing_mode: AddressingMode::AddressingAbsolute},
		Ins6502 {opcode: 0x7E, mnem: "ROR".to_string(), addressing_mode: AddressingMode::AddressingAbsoluteX},
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


    fn get_operand(self: &mut Self, instruction_index: u8) -> u16 {
	let mode = &self.ins[instruction_index as usize].addressing_mode;

	match mode {
	    AddressingMode::AddressingImplied	=> { return 0; } 

	    AddressingMode::AddressingImmediate	=> {
		let immediate: u16 = self.regs.pc;
		self.regs.pc += 1;
		immediate
	    }

	    AddressingMode::AddressingIndirect	=> {
		let lo: u16 = self.regs.pc;
		let hi: u16 = self.regs.pc + 1;
		let addr: u16 = (hi << 4) as u16 | lo as u16;

		self.regs.pc += 2;
		addr
	    }

	    AddressingMode::AddressingIndirectX	=> {
		let lo: u16 = (self.bus.vram[self.regs.pc as usize] + self.regs.x).into();
		let addr: u16 = self.bus.vram[lo as usize].into();
		self.regs.pc += 1;
		addr
	    }

	    AddressingMode::AddressingIndirectY	=> {
		let lo: u16 = (self.bus.vram[self.regs.pc as usize] + self.regs.y).into();
		let addr: u16 = self.bus.vram[lo as usize].into();
		self.regs.pc += 1;
		addr
	    }

	    AddressingMode::AddressingAbsolute	=> {
		let absolute: u16 = (self.bus.vram[(self.regs.pc + 1) as usize] << 4) as u16
		    | (self.bus.vram[(self.regs.pc + 2) as usize] as u16);
		self.regs.pc += 2;
		absolute
	    }

	    AddressingMode::AddressingAbsoluteX	=> {
		let mut absolute: u16 = (self.bus.vram[(self.regs.pc) as usize] << 4) as u16
		    | (self.bus.vram[(self.regs.pc + 1) as usize] as u16);
		absolute = (((absolute & 0xFF00) >> 4) as u8 + self.regs.x) as u16 | (absolute & 0xFF) as u16;
		self.regs.pc += 2;
		absolute
	    }

	    AddressingMode::AddressingAbsoluteY	=> {
		let mut absolute: u16 = (self.bus.vram[(self.regs.pc) as usize] << 4) as u16
		    | (self.bus.vram[(self.regs.pc + 1) as usize] as u16);
		absolute = (((absolute & 0xFF00) >> 4) as u8 + self.regs.y) as u16 | (absolute & 0xFF) as u16;
		self.regs.pc += 2;
		absolute
	    }

	    AddressingMode::AddressingZeroPage	=> {
		let zp: u16 = self.bus.vram[self.regs.pc as usize] as u16;
		self.regs.pc += 1;
		zp
	    }

	    AddressingMode::AddressingZeroPageX	=> {
		let zp: u16 = (self.bus.vram[self.regs.pc as usize] + self.regs.x)
		    .try_into()
		    .unwrap();
		self.regs.pc += 1;
		zp
	    }

	    _ => todo!()
	}
    }

    fn instruction_fetch(self: &mut Self) -> u8 {
	let instruction = self.bus.vram[self.regs.pc as usize];
	self.regs.pc = self.regs.pc + 1;
	instruction
    }

    fn instruction_execute(self: &mut Self, opcode: u8) -> i32{
	// https://www.masswerk.at/6502/6502_instruction_set.html
	let index_of_ins_in_vec = self.ins.iter().position(|ins| ins.opcode == opcode).unwrap();
	let operand = self.get_operand(index_of_ins_in_vec as u8);
	let ins: &Ins6502 = &self.ins[index_of_ins_in_vec];

	println!("{}, {:#x} {:#x} ({})", ins.mnem, operand, self.bus.vram[operand as usize], ins.addressing_mode.to_string());

	match &ins.mnem as &str {
	    "AND" => { self.regs.a = self.regs.a & self.bus.vram[operand as usize]; }
	    "ASL" => { self.bus.vram[operand as usize] = self.bus.vram[operand as usize] << 1; }

	    "ORA" => {
		self.regs.a = self.regs.a | operand as u8;
		self.set_status_bit('a', 'N');
		self.set_status_bit('a', 'Z');
	    }

	    "LDA" => { // there's gotta be a better way to do this!
		self.regs.a = self.bus.vram[operand as usize] as u8;
		self.set_status_bit('a', 'N');
		self.set_status_bit('a', 'Z');
	    }

	    "CMP" => {
		let v: u8 = self.bus.vram[operand as usize];
		if self.regs.a > v { self.regs.p = self.regs.p | STATUS_FLAG_C; }
	    }

	    "PHA" => { self.bus.vram[(MEMSTART_STACK_6502 - self.regs.sp as u16) as usize] = self.regs.a; self.regs.sp -= 1; }
	    "PHP" => { self.bus.vram[(MEMSTART_STACK_6502 - self.regs.sp as u16) as usize] = self.regs.p; self.regs.sp -= 1; }
	    "PLA" => { self.regs.a = self.bus.vram[(MEMSTART_STACK_6502 - self.regs.sp as u16) as usize]; self.regs.sp += 1; }
	    "PLP" => { self.regs.p = self.bus.vram[(MEMSTART_STACK_6502 - self.regs.sp as u16) as usize]; self.regs.sp += 1; }

	    "ROL" => { self.bus.vram[operand as usize] = self.bus.vram[operand as usize].rotate_left(1); }
	    "ROR" => { self.bus.vram[operand as usize] = self.bus.vram[operand as usize].rotate_right(1); }

	    "NOP" => { println!("!NOP!"); }

	    "JMP" => { self.regs.pc = operand; }
	    "STX" => { self.bus.vram[operand as usize] = self.regs.x; }
	    "STY" => { self.bus.vram[operand as usize] = self.regs.y; }
	    "STA" => { self.bus.vram[operand as usize] = self.regs.a; }

	    "BRK" => { return 0; }

	    "CLC" => { self.regs.p = self.regs.p & !STATUS_FLAG_C; }
	    "CLD" => { self.regs.p = self.regs.p & !STATUS_FLAG_D; }
	    "CLI" => { self.regs.p = self.regs.p & !STATUS_FLAG_I; }
	    "CLV" => { self.regs.p = self.regs.p & !STATUS_FLAG_V; }

	    "TAX" => { self.regs.x = self.regs.a; }
	    "TAY" => { self.regs.y = self.regs.a; }
	    "TSX" => { self.regs.x = self.regs.sp; }
	    "TXA" => { self.regs.a = self.regs.x; }
	    "TXS" => { self.regs.sp = self.regs.x;}
	    "TYA" => { self.regs.a = self.regs.y;}

	    
	    "LDX" => { self.regs.x = self.bus.vram[operand as usize] as u8; }
	    "LDY" => { self.regs.y = self.bus.vram[operand as usize] as u8; }

	    "INC" => { self.bus.vram[operand as usize] = self.bus.vram[operand as usize] + 1 }
	    "INX" => { self.regs.x = self.regs.x + 1 }
	    "INY" => { self.regs.y = self.regs.y + 1 }

	    _ => todo!()
	}
	return 1;
    }

    fn run(self: &mut Self){
	println!("Executing...\n");
	loop {
	    let op = self.instruction_fetch();
	    let status = self.instruction_execute(op);
	    if status == 0 {
		return;
	    }
	}
    }

    fn load(self: &mut Self, rom_buff: &Vec<u8>){
	self.bus.vram[MEMSTART_CARTRIDGE_ROM_6502 as usize .. (MEMSTART_CARTRIDGE_ROM_6502 as usize + rom_buff.len())].copy_from_slice(&rom_buff[..]);
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
    let rom_buff;

    if args.len() > 1 {
	rom_filename = &args[1];
	println!("Reading contents of {}...", rom_filename);
	rom_buff = file_to_u8_vector(rom_filename);
    }
    else {
	rom_buff = vec!(0xa9, 0x05, 0x00);
    }

    let mut cpu = Cpu6502::new();
    cpu.load(&rom_buff);

    println!("Entering CPU loop!");

    cpu.run();
    println!("Done! See ya.");
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec!(0xa9, 0x05, 0x00);

	cpu.load(&rom_buff);
	cpu.run();
	println!("{:b}", cpu.regs.p);
	assert_eq!(cpu.regs.a, 0x05);
	assert!(cpu.regs.p & STATUS_FLAG_Z == 0b00);
	assert!(cpu.regs.p & STATUS_FLAG_N == 0b00);
    }

    #[test]
    fn test_0xa9_lda_immediate_zero_flag() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa9, 0x00, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.p & STATUS_FLAG_Z == 0b10);
    }

    #[test]
    fn test_0xa5_lda_zp() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec!(0xa2, 0x2a, // load 42 into x
				0x86, 0x01, // load x at mem[1]
				0xa5, 0x01, // load mem[0] into a
				0x00);

	cpu.load(&rom_buff);
	cpu.run();
	println!("{:b}", cpu.regs.p);
	assert!(cpu.regs.a == 0x2a);
    }

    #[test]
    fn test_0xb5_lda_zpx() {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec!(0xa2, 0x2a, // load 42 into x
				0x86, 0x01, // load x at mem[1]
				0xa2, 0x00, // load 42 into x
				0xB5, 0x01, // load mem[1] into a
				0x00);

	cpu.load(&rom_buff);
	cpu.run();
	println!("{:b}", cpu.regs.p);
	assert!(cpu.regs.a == 0x2a);
    }

    #[test]
    fn test_0xa2_ldx_immediate () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa2, 0xc0, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.x == 0xc0);
    }

    #[test]
    fn test_0xa0_ldy_immediate () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa0, 0xc0, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.y == 0xc0);
    }

    #[test]
    fn test_0x8a_txa_implied () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa2, 0xc0, 0x8a, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.a == 0xc0);
    }

    #[test]
    fn test_0xaa_tax_implied () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa9, 0xc0, 0xaa, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.x == 0xc0);
    }

    #[test]
    fn test_0x86_stx_zp () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa2, 0x03, 0x86, 0x01, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.bus.vram[1] == 0x03);
    }

    #[test]
    fn test_0x84_sty_zp () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa0, 0x03, 0x84, 0x01, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.bus.vram[1] == 0x03);
    }

    #[test]
    fn test_0x85_sta_zp () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa9, 0x03, 0x85, 0x01, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.bus.vram[1] == 0x03);
    }

    #[test]
    fn test_0x95_sta_zpx () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa9, 0x03, // lda 3
				0xa2, 0x03, // ldx 3
				0x95, 0x01, // sta mem[x + 1]
				0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.bus.vram[4] == 0x03);
    }

    // #[test]
    // fn test_0x8d_sta_abs () {
    // 	let mut cpu = Cpu6502::new();
    // 	let mut rom_buff = vec![0xa9, 0x03, // lda 3
    // 				0xa2, 0x03, // ldx 3
    // 				0x8D, 0x01, // sta mem[x + 1]
    // 				0x00];

    // 	cpu.load(&rom_buff);
    // 	cpu.run();
    // 	assert!(cpu.bus.vram[4] == 0x03);
    // }

    #[test]
    fn test_0x01_ora_ind_x () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xa2, 0x01, // load 1 into x     
				0x86, 0x01, // write x to mem[1]
				0xa2, 0x02, // load 2 into x
				0x86, 0x24, // write x to mem[24]
				0xa2, 0x04, // load 4 into X
				0xa9, 0x01, // load 1 in a
				0x01, 0x20, // a -> a | mem[mem[x + 20]]
				0x00];	    // brk

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.a == 0x3);
    }

    #[test]
    fn test_e8_c8_inxy_implied () {
	let mut cpu = Cpu6502::new();
	let mut rom_buff = vec![0xe8, 0xc8, 0x00];

	cpu.load(&rom_buff);
	cpu.run();
	assert!(cpu.regs.x == 0x1);
	assert!(cpu.regs.y == 0x1);
    }
}
