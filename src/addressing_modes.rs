
// 📝: non-indexed addressing

fn	addressing_immediate(instruction_buffer: &Vec<u8>, pc: &mut u16) -> u8 {
}
fn	addressing_absolute(instruction_buffer: &Vec<u8>, pc: &mut u16) -> u16 {
}

fn	addressing_zero_page(vram: &[u8; MEMSIZE_6502 as usize], instruction_buffer: &Vec<u8>, pc: &mut u16) -> u8 {
o}

fn	addressing_indirect(vram: &[u8; MEMSIZE_6502 as usize], instruction_buffer: &Vec<u8>, pc: &mut u16) -> u8 {
    let addr: u16 = addressing_absolute(instruction_buffer, pc) as u16;
    let val: u16 = vram[addr as usize] as u16;
    val
}

// 📝 : indexed addressing
fn	addressing_immediate_x(instruction_buffer: &Vec<u8>, pc: &mut u16) -> u8 {
    let immediate: u8 = instruction_buffer[(*pc + 1) as usize];
    *pc += 1;
    immediate
}

fn	addressing_absolute_x(regs: &Regs6502, instruction_buffer: &Vec<u8>, pc: &mut u16) -> u16 {
    let absolute: u16 = (instruction_buffer[(*pc) as usize] << 8) as u16
	| (instruction_buffer[(*pc + 1) as usize] as u16);
    absolute = absolute + regs.x;
    *pc += 2;
    absolute
}

fn	addressing_zero_page_x(vram: &[u8; MEMSIZE_6502 as usize], instruction_buffer: &Vec<u8>, pc: &mut u16) -> u16 {
    let zp: u16 = addressing_immediate(instruction_buffer, pc) as u16;
    let val: u16 = (0x00 << 8) as u16 | vram[zp as usize] as u16;
    *pc += 2;
    val
}

fn	addressing_immediate_y(instruction_buffer: &Vec<u8>, pc: &mut u16) -> u8 {
    let immediate: u8 = instruction_buffer[(*pc + 1) as usize];
    *pc += 1;
    immediate
}

fn	addressing_absolute_y(instruction_buffer: &Vec<u8>, pc: &mut u16) -> u16 {
    let absolute: u16 = (instruction_buffer[(*pc + 1) as usize] << 8) as u16
	| (instruction_buffer[(*pc + 2) as usize] as u16);
    *pc += 2;
    absolute
}

fn	addressing_zero_page_y(vram: &[u8; MEMSIZE_6502 as usize], instruction_buffer: &Vec<u8>, pc: &mut u16) -> u16 {
    let zp: u16 = addressing_immediate(instruction_buffer, pc) as u16;
    let val: u16 = (0x00 << 8) as u16 | vram[zp as usize] as u16;
    *pc += 2;
    val
}
