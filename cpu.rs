
use std::io::println;
use mem::Mem;
struct Reg {
	v : u16
}

impl Reg {
	fn set_high(&mut self, v : u8) {
		self.v = (v as u16 << 8) | (self.v & 0xFF) as u16;
	}
	fn set_low(&mut self, v : u8) {
		self.v = (self.v & 0xFF00) as u16 | v as u16
	}
	fn get_high(&self) -> u8 {
		(self.v >> 8) as u8
	}
	fn get_low(&self) -> u8 {
		(self.v & 0xFF) as u8
	}
	fn inc_high(&mut self) {
		let o = self.get_high()+1;
		self.set_high(o);
	}
	fn inc_low(&mut self) {
		let o = self.get_low()+1;
		self.set_low(o);
	}
	fn dec_high(&mut self) {
		let o = self.get_high()-1;
		self.set_high(o);
	}
	fn dec_low(&mut self) {
		let o = self.get_low()-1;
		self.set_low(o);
	}
	fn add_high(&mut self, v : u8) {
		let o = self.get_high()+v;
		self.set_high(o);
	}
	fn add_low(&mut self, v : u8) {
		let o = self.get_low()+v;
		self.set_low(o);
	}
	fn sub_high(&mut self, v : u8) {
		let o = self.get_high()-v;
		self.set_high(o);
	}
	fn sub_low(&mut self, v : u8) {
		let o = self.get_low()-v;
		self.set_low(o);
	}
	fn to_bytes(&self) -> ~[u8] {
		~[self.get_high(), self.get_low()]
	}
}

struct Regs {
	af : Reg,
	bc : Reg,
	de : Reg,
	hl : Reg,
	sp : Reg,
	pc : Reg
}

impl Regs {
	fn new() -> Regs {
		Regs {
		   af: Reg { v: 0x01B0 },
		   bc: Reg { v: 0x0013 },
		   de: Reg { v: 0x00D8 },
		   hl: Reg { v: 0x014D },
		   sp: Reg { v: 0xFFFE },
		   pc: Reg { v: 0x0100 },
		}
	}
}

pub struct Cpu {
	regs : Regs,
	mem : Mem
}

impl Cpu {
	pub fn new(rom : ~[u8]) -> Cpu {
		Cpu {
			regs : Regs::new(),
			mem : Mem { bank_n : 1, mem : [0, ..0x10000], rom : rom },
		}
	}
	fn push(&mut self, v : u8) {
		self.regs.sp.v -= 1;
		self.mem.writebyte(self.regs.sp.v, v);
	}
	fn push_16(&mut self, v : u16) {
		self.regs.sp.v -= 2;
		let m : ~[u8] = ~[(v & 0xFF) as u8, (v >> 8) as u8];
		self.mem.write(self.regs.sp.v, m);
		//println!("{:02X}{:02X}", self.mem.readbyte(self.regs.sp.v+1), self.mem.readbyte(self.regs.sp.v))
	}
	fn pop(&mut self) -> u8 {
		let r = self.mem.readbyte(self.regs.sp.v);
		self.regs.sp.v += 1;
		r
	}
	fn pop_16(&mut self) -> u16 {
		let mut r = self.mem.readbyte(self.regs.sp.v) as u16;
		self.regs.sp.v += 1;
		r |= self.mem.readbyte(self.regs.sp.v) as u16 << 8;
		self.regs.sp.v += 1;
		r
	}
	fn call(&mut self, v : u16) {
		self.push_16(self.regs.pc.v+1);
		self.regs.pc.v = v-1;
	}
	fn ret(&mut self) {
		self.regs.pc.v = self.pop_16()-1;
	}
	pub fn run(&mut self) {
		fn sign(v : u8) -> i8 {
			// Dunno how to cast to signed :S
			if (v & 0x80) == 0x80 {
				-((!v)+1) as i8
			} else {
				v as i8
			}
		}
		let mut count = 0;
		loop {
			println("");
			let op : u8 = self.mem.readbyte(self.regs.pc.v);
			let n : u8 = self.mem.readbyte(self.regs.pc.v+1);
			let nn : u16 = n as u16 | self.mem.readbyte(self.regs.pc.v+2) as u16 << 8;
			//println!("PC: {:04X} OP: {:02X} N: {:02X} NN: {:04X} SP: {:04X} AF: {:04X} BC: {:04X} DE: {:04X} HL: {:04X} On Stack: {:04X}",
			println!("{:04X} {:02X} {:02X} {:02X}\t\tSP: {:04X} AF: {:04X} BC: {:04X} DE: {:04X} HL: {:04X} On Stack: {:04X}",
					 self.regs.pc.v, op, n, nn>>8, self.regs.sp.v,
					 self.regs.af.v, self.regs.bc.v, self.regs.de.v, self.regs.hl.v,
					 self.mem.read16(self.regs.sp.v));
			println!("-6 {:04X} -4 {:04X} -2 {:04X} +0 {:04X} +2 {:04X} +4 {:04X}",
					 self.mem.read16(self.regs.sp.v-6),
					 self.mem.read16(self.regs.sp.v-4),
					 self.mem.read16(self.regs.sp.v-2),
					 self.mem.read16(self.regs.sp.v),
					 self.mem.read16(self.regs.sp.v+2),
					 self.mem.read16(self.regs.sp.v+4));
			match op {
				0x00 => {},
				0x01 => {self.regs.bc.v = nn; self.regs.pc.v += 2},
				0x02 => {self.mem.writebyte(self.regs.bc.v, self.regs.af.get_high())},
				0x03 => {self.regs.bc.v += 1},
				0x04 => {self.regs.bc.inc_high()},
				0x05 => {self.regs.bc.dec_high()},
				0x06 => {self.regs.bc.set_high(n); self.regs.pc.v += 1},
				0x07 => {fail!("TODO")},
				0x08 => {self.mem.write(nn, self.regs.sp.to_bytes()); self.regs.pc.v += 2},
				0x09 => {self.regs.hl.v += self.regs.bc.v},
				0x0A => {self.regs.af.set_high(self.mem.readbyte(self.regs.bc.v))},
				0x0B => {self.regs.bc.v -= 1},
				0x0C => {self.regs.bc.inc_low()},
				0x0D => {self.regs.bc.dec_low()},
				0x0E => {self.regs.bc.set_low(n); self.regs.pc.v += 1},
				0x0F => {fail!("TODO")},
				0x10 => {fail!("STOP")},
				0x11 => {self.regs.de.v = nn; self.regs.pc.v += 2},
				0x12 => {self.mem.writebyte(self.regs.de.v, self.regs.af.get_high())},
				0x13 => {self.regs.de.v += 1},
				0x14 => {self.regs.de.inc_high()},
				0x15 => {self.regs.de.dec_high()},
				0x21 => {self.regs.pc.v += 2; self.regs.hl.v = nn},
				0x23 => {self.regs.hl.v += 1},
				//0x => {},

				0x31 => {self.regs.sp.v = nn; self.regs.pc.v += 2},
				0x39 => {self.regs.hl.v += self.regs.sp.v},
				0x3E => {self.regs.af.set_high(n); self.regs.pc.v += 1},
				
				0x66 => {self.regs.hl.set_high(self.mem.readbyte(self.regs.hl.v))},
				0x6F => {self.regs.hl.set_low(self.regs.af.get_high())},

				0x7D => {self.regs.af.set_high(self.regs.hl.get_low())},
				0x7E => {self.regs.af.set_high(self.mem.readbyte(self.regs.hl.v))},

				0x97 => {self.regs.af.set_high(0)},
				0xC1 => {self.regs.bc.v = self.pop_16()},
				0xC3 => {self.regs.pc.v = nn-1},
				0xC5 => {self.push_16(self.regs.bc.v)},
				0xC9 => {self.ret()},
				0xCB => {
					println("WARNING: CB prefix instruction");
					let f = if n < 0x8 { // RLC
						|x: u8| {((x << 1) | (x >> 7))}
					} else if n < 0x10 { // RRC
						|x: u8| {((x >> 1) | (x << 7))}
					} else if n < 0x18 { // RL
						|x: u8| {((x << 1) | (x >> 7))}
					} else if n < 0x20 { // RR
						|x: u8| {((x >> 1) | (x << 7))}
					} else if n < 0x28 { // SLA
						|x: u8| {x << 1}
					} else if n < 0x30 { // SRA
						|x: u8| {x >> 1}
					} else if n < 0x38 { // SWAP
						|x: u8| {x << 8 | x >> 8}
					} else if n < 0x40 { // SRL
						|x: u8| {x >> 1}
					} else if n < 0x80 { // BIT
						|x: u8| {let b = n >> 3; (x >> b) & 1}
					} else if n < 0xC0 { // RES
						|x: u8| {let b = ((n >> 3) & 0xF)-1; x & (0xFF ^ (1 << b))}
					} else { // SET
						|x: u8| {let b = ((n >> 3) & 0xF)-1; x | (1 << b)}
					};
					match n & 7 {
						0 => {let x = self.regs.bc.get_high(); self.regs.bc.set_high(f(x))},
						1 => {let x = self.regs.bc.get_low(); self.regs.bc.set_low(f(x))},
						2 => {let x = self.regs.de.get_high(); self.regs.de.set_high(f(x))},
						3 => {let x = self.regs.de.get_low(); self.regs.de.set_low(f(x))},
						4 => {let x = self.regs.hl.get_high(); self.regs.hl.set_high(f(x))},
						5 => {let x = self.regs.hl.get_low(); self.regs.hl.set_low(f(x))},
						6 => {let a = self.regs.hl.v;
								let x = self.mem.readbyte(a); self.mem.writebyte(a, f(x))},
						7 => {let x = self.regs.af.get_high(); self.regs.af.set_high(f(x))},
						_ => fail!("wat.")
					}
					self.regs.pc.v += 1;
				},
				0xCD => {self.regs.pc.v += 2; self.call(nn)},
				0xD1 => {self.regs.de.v = self.pop_16()},
				0xD5 => {self.push_16(self.regs.de.v)},
				0xE0 => {
					let mut addr : u16 = 0xFF00;
					addr += n as u16;
					self.mem.writebyte(addr, self.regs.af.get_high());
					self.regs.pc.v += 1;
				},
				0xE5 => {self.push_16(self.regs.hl.v)},
				0xE8 => {
					let r = (self.regs.sp.v as i16 + sign(n) as i16) as u16;
					self.regs.sp.v = r;
					self.regs.pc.v += 1;},
				0xE9 => { // Docs says its a jump to (hl), but seems it's jp hl
					//let a = self.regs.hl.v; self.regs.pc.v = self.mem.read16(a)-1},
					self.regs.pc.v = self.regs.hl.v-1},
				0xEA => {
					self.mem.writebyte(nn, self.regs.af.get_high());
					self.regs.pc.v += 2;
				},
				0xF3 => {println("WARNING: DI")},
				_ => {fail!("Unimplemented OP: {:X}h", op)},
			}

			self.regs.pc.v += 1;
			if count > 60 {
				break;
			}
			count += 1;
		}
	}
}

