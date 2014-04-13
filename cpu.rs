
use std::io::println;
use mem::Mem;
struct Reg {
	v : u16
}

impl Reg {
	fn set_high(&mut self, v : u8) {
		self.v = (v << 8) as u16 | (self.v & 0xFF) as u16
	}
	fn set_low(&mut self, v : u8) {
		self.v = (self.v & 0xFF00) as u16 | v as u16
	}
	fn get_high(&self) -> u8 {
		(self.v & 0xFF00) as u8
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
		self.push_16(self.regs.pc.v);
		self.regs.pc.v = v-1;
	}
	fn ret(&mut self) {
		self.regs.pc.v = self.pop_16();
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
			let op : u8 = self.mem.readbyte(self.regs.pc.v);
			let n : u8 = self.mem.readbyte(self.regs.pc.v+1);
			let nn : u16 = n as u16 | self.mem.readbyte(self.regs.pc.v+2) as u16 << 8;
			println!("PC: {:04X}\tOP: {:02X}\tN: {:02X}\tNN: {:04X}\tSP: {:04X}",
				self.regs.pc.v, op, n, nn, self.regs.sp.v);
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
				0x39 => {self.regs.sp.v += self.regs.hl.v},
				0x3E => {self.regs.af.set_high(n); self.regs.pc.v += 1},
				
				0x66 => {self.regs.hl.set_high(self.mem.readbyte(self.regs.hl.v))},
				0x6F => {self.regs.hl.set_low(self.regs.af.get_high())},

				0x7D => {self.regs.af.set_high(self.regs.hl.get_low())},
				0x7E => {self.regs.af.set_high(self.mem.readbyte(self.regs.hl.v))},

				0x97 => {self.regs.af.set_high(0)},
				0xC3 => {self.regs.pc.v = nn-1},
				0xC9 => {self.ret()},
				0xCD => {self.regs.pc.v += 2; self.call(nn)},
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

