

// This, like any cpu emulator, is a fucking mess.
use std::io::File;
use std::io::println;
use mem::Mem;
struct Reg {
	v : u16
}

fn sign(v : u8) -> i8 {
	// Dunno how to cast to signed :S
	if (v & 0x80) == 0x80 {
		-((!v)+1) as i8
	} else {
		v as i8
	}
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

	fn inc_high(&mut self) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o+1);
		(o, o+1)
	}
	fn inc_low(&mut self) -> (u8, u8) {
		let o = self.get_low();
		self.set_low(o+1);
		(o, o+1)
	}
	fn inc(&mut self) -> (u16, u16) {
		let o = self.v;
		self.v = o+1;
		(o, o+1)
	}
	fn dec_high(&mut self) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o-1);
		(o, o-1)
	}
	fn dec_low(&mut self) -> (u8, u8) {
		let o = self.get_low();
		self.set_low(o-1);
		(o, o-1)
	}
	fn dec(&mut self) -> (u16, u16) {
		let o = self.v;
		self.v = o-1;
		(o, o-1)
	}
	fn add_high(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o+v);
		(o, o+v)
	}
	fn add_low(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_low();
		self.set_low(o+v);
		(o, o+v)
	}
	fn add(&mut self, v : u16) -> (u16, u16) {
		let o = self.v;
		self.v = o+v;
		(o, o+v)
	}
	fn sub_high(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o-v);
		(o, o-v)
	}
	fn sub_low(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_low();
		self.set_low(o-v);
		(o, o-v)
	}
	fn sub(&mut self, v : u16) -> (u16, u16) {
		let o = self.v;
		self.v = o-v;
		(o, o-v)
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
	pub mem : Mem,
	clock : uint,
	screen_mode : int,
	pub drawing : bool
}

impl Cpu {
	pub fn new(rom : ~[u8]) -> Cpu {
		Cpu {
			regs : Regs::new(),
			mem : Mem { bank_n : 1, mbc_type : 0, mem : [0, ..0x10000], rom : rom },
			clock : 0,
			screen_mode : 0,
			drawing : false
		}
	}
	// Tests for carry
	fn ca8(&mut self, (old, new) : (u8, u8)) -> u8 {
		self.set_carry_flag(new < old);
		new
	}
	fn ca16(&mut self, (old, new) : (u16, u16)) -> u16 {
		self.set_carry_flag(new < old);
		new
	}
	// subtraction
	fn cs8(&mut self, (old, new) : (u8, u8)) -> u8 {
		self.set_carry_flag(new > old);
		new
	}
	fn cs16(&mut self, (old, new) : (u16, u16)) -> u16 {
		self.set_carry_flag(new > old);
		new
	}
	// Tests for zero in 8bit registers
	fn z8(&mut self, val : u8) {
		self.set_zero_flag(val == 0);
	}

	fn z16(&mut self, val : u16) {
		self.set_zero_flag(val == 0);
	}

	fn incflags(&mut self, t : (u8, u8)) {
		let r = self.ca8(t);
		self.z8(r);
		self.set_addsub_flag(false);
	}

	fn incflags16(&mut self, t : (u16, u16)) {
		let r = self.ca16(t);
		self.z16(r);
		self.set_addsub_flag(false);
	}

	fn addflags(&mut self, t : (u8, u8)) {
		self.incflags(t);
		self.set_addsub_flag(true);
	}

	fn addflags16(&mut self, t : (u16, u16)) {
		self.incflags16(t);
		self.set_addsub_flag(true);
	}

	fn decflags(&mut self, t : (u8, u8)) {
		let r = self.cs8(t);
		self.z8(r);
		self.set_addsub_flag(false);
	}

	fn decflags16(&mut self, t : (u16, u16)) {
		let r = self.cs16(t);
		self.z16(r);
		self.set_addsub_flag(false);
	}

	fn subflags(&mut self, t : (u8, u8)) {
		self.decflags(t);
		self.set_addsub_flag(true);
	}

	fn subflags16(&mut self, t : (u16, u16)) {
		self.decflags16(t);
		self.set_addsub_flag(true);
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
	fn check_carry_flag(&mut self) -> bool {
		self.regs.af.get_low() & (1 << 4) != 0
	}
	fn set_carry_flag(&mut self, v : bool) {
		let n = if v {
			self.regs.af.get_low() | (1 << 4)
		} else {
			self.regs.af.get_low() & !(1 << 4)
		};
		self.regs.af.set_low(n);
	}
	fn check_zero_flag(&mut self) -> bool {
		self.regs.af.get_low() & (1 << 7) != 0
	}
	fn set_zero_flag(&mut self, v : bool) {
		let n = if v {
			self.regs.af.get_low() | (1 << 7)
		} else {
			self.regs.af.get_low() & !(1 << 7)
		};
		self.regs.af.set_low(n);
	}
	fn set_addsub_flag(&mut self, v : bool) {
		let n = if v {
			self.regs.af.get_low() | (1 << 6)
		} else {
			self.regs.af.get_low() & !(1 << 6)
		};
		self.regs.af.set_low(n);
	}
	fn set_hc_flag(&mut self, v : bool) {
		let n = if v {
			self.regs.af.get_low() | (1 << 5)
		} else {
			self.regs.af.get_low() & !(1 << 5)
		};
		self.regs.af.set_low(n);
	}
	fn and(&mut self, v : u8) {
		let a = self.regs.af.get_high() & v;
		self.set_zero_flag(a == 0);
		self.regs.af.set_high(a)
	}
	fn or(&mut self, v : u8) {
		let a = self.regs.af.get_high() | v;
		self.set_zero_flag(a == 0);
		self.regs.af.set_high(a)
	}
	fn xor(&mut self, v : u8) {
		let a = self.regs.af.get_high() ^ v;
		self.set_zero_flag(a == 0);
		self.regs.af.set_high(a)
	}
	fn cp(&mut self, v : u8) {
		let a = self.regs.af.get_high();
		self.set_zero_flag(a == v);
		self.set_carry_flag(a < v);
	}
	fn jr(&mut self, v : u8) {
		self.regs.pc.v = (self.regs.pc.v as i16 + sign(v) as i16) as u16 + 1;
	}
	fn halt(&mut self) {
		fail!("halt, unimplemented")
	}
	fn run_clock(&mut self) {
		self.clock += 4; // TODO: precise cycles
		let line = &mut self.mem.mem[0xff44];
		match self.screen_mode {
			0 => {
				if self.clock >= 204 {
					self.clock = 0;
					*line += 1;
					if *line == 143 {
						self.drawing = true;
						self.screen_mode = 1;
					} else {
						self.screen_mode = 2;
					}
				}
			},
			1 => {
				if self.clock >= 456 {
					self.clock = 0;
					*line += 1;
					if *line > 153 {
						self.screen_mode = 2;
						*line = 0;
					}
				}
			},
			2 => {
				if self.clock >= 80 {
					self.clock = 0;
					self.screen_mode = 3;
				}
			},
			3 => {
				if self.clock >= 172 {
					self.clock = 0;
					self.screen_mode = 0;
				}
			},
			_ => fail!("Wat"),
		}
	}
	pub fn next(&mut self) {
		//if self.regs.pc.v == 0x03C6 {
		//	let mut file = File::create(&Path::new("ram_dump.bin"));
		//	file.write(self.mem.mem);
		//	fail!("quit")
		//}
		//let mut count = 0;
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
				 self.mem.read16(self.regs.hl.v-6),
				 self.mem.read16(self.regs.hl.v-4),
				 self.mem.read16(self.regs.hl.v-2),
				 self.mem.read16(self.regs.hl.v),
				 self.mem.read16(self.regs.hl.v+2),
				 self.mem.read16(self.regs.hl.v+4));
		match op {
			0x00 => {},
			0x01 => {self.regs.bc.v = nn; self.regs.pc.v += 2},
			0x02 => {self.mem.writebyte(self.regs.bc.v, self.regs.af.get_high())},
			0x03 => {self.regs.bc.v += 1},
			0x04 => {let a = self.regs.bc.inc_high(); self.incflags(a)},
			0x05 => {let a = self.regs.bc.dec_high(); self.decflags(a)},
			0x06 => {self.regs.bc.set_high(n); self.regs.pc.v += 1},
			0x07 => {fail!("TODO: RLCA")},
			0x08 => {self.mem.write(nn, self.regs.sp.to_bytes()); self.regs.pc.v += 2},
			0x09 => {self.regs.hl.v += self.regs.bc.v},
			0x0A => {self.regs.af.set_high(self.mem.readbyte(self.regs.bc.v))},
			0x0B => {self.regs.bc.v -= 1},
			0x0C => {let a = self.regs.bc.inc_low(); self.incflags(a)},
			0x0D => {let a = self.regs.bc.dec_low(); self.decflags(a)},
			0x0E => {self.regs.bc.set_low(n); self.regs.pc.v += 1},
			0x0F => {fail!("TODO")},
			0x10 => {fail!("STOP")},
			0x11 => {self.regs.de.v = nn; self.regs.pc.v += 2},
			0x12 => {self.mem.writebyte(self.regs.de.v, self.regs.af.get_high())},
			0x13 => {self.regs.de.v += 1},
			0x14 => {let a = self.regs.de.inc_high(); self.incflags(a)},
			0x15 => {let a = self.regs.de.dec_high(); self.decflags(a)},
			0x16 => {self.regs.de.set_high(n); self.regs.pc.v += 1},
			0x18 => self.jr(n),
			0x1B => {let f = self.regs.de.dec(); self.decflags16(f)},
			0x1E => {self.regs.de.set_low(n); self.regs.pc.v += 1},
			0x1D => {let a = self.regs.de.dec_low(); self.decflags(a)},
			0x20 => if !self.check_zero_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x21 => {self.regs.pc.v += 2; self.regs.hl.v = nn},
			0x22 => {
				let addr = self.regs.hl.v;
				self.mem.writebyte(addr, self.regs.af.get_high());
				self.regs.hl.v += 1},
			0x23 => {self.regs.hl.v += 1},
			0x28 => if self.check_zero_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x2B => {let f = self.regs.hl.dec(); self.decflags16(f)},
			0x2F => {let a = self.regs.af.get_high(); self.regs.af.set_high(a^0xFF);
				self.set_addsub_flag(true); self.set_hc_flag(true)},

			0x30 => if !self.check_carry_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x31 => {self.regs.sp.v = nn; self.regs.pc.v += 2},
			0x36 => {let addr = self.regs.hl.v;
				self.mem.writebyte(addr, n);
				self.regs.pc.v += 1},
			0x38 => if self.check_carry_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x39 => {self.regs.hl.v += self.regs.sp.v},
			0x3B => {let f = self.regs.sp.dec(); self.decflags16(f)},
			0x3D => {let a = self.regs.af.dec_high(); self.decflags(a)},
			0x3E => {self.regs.af.set_high(n); self.regs.pc.v += 1},
			
			0x66 => {self.regs.hl.set_high(self.mem.readbyte(self.regs.hl.v))},
			0x6F => {self.regs.hl.set_low(self.regs.af.get_high())},

			0x77 => {let addr = self.regs.hl.v;
				self.mem.writebyte(addr, self.regs.af.get_high())},
			0x7D => {self.regs.af.set_high(self.regs.hl.get_low())},
			0x7E => {self.regs.af.set_high(self.mem.readbyte(self.regs.hl.v))},

			0x40..0xBF => {
				let b = match op & 0x7 {
					0 => self.regs.bc.get_high(),
					1 => self.regs.bc.get_low(),
					2 => self.regs.de.get_high(),
					3 => self.regs.de.get_low(),
					4 => self.regs.hl.get_high(),
					5 => self.regs.hl.get_low(),
					6 => self.mem.readbyte(self.regs.hl.v),
					7 => self.regs.af.get_high(),
					_ => fail!("wat")
				};
				match op {
					0x40..0x47 => self.regs.bc.set_high(b),
					0x48..0x4F => self.regs.bc.set_low(b),
					0x50..0x57 => self.regs.de.set_high(b),
					0x58..0x5F => self.regs.de.set_low(b),
					0x60..0x67 => self.regs.hl.set_high(b),
					0x68..0x6F => self.regs.hl.set_low(b),
					0x70..0x77 => if op == 0x76 {self.halt()}
						else {self.mem.writebyte(self.regs.hl.v, b)},
					0x78..0x7F => self.regs.af.set_high(b),
					0x80..0x87 => {
						let f = self.regs.af.add_high(b);
						self.addflags(f)},
					0x88..0x8F => { // FIXME!: carry
						let f = self.regs.af.add_high(b);
						self.addflags(f)},
					0x90..0x97 => {
						let f = self.regs.af.sub_high(b);
						self.subflags(f)},
					0x98..0x9F => { // FIXME!: carry
						let f = self.regs.af.sub_high(b);
						self.subflags(f)},
					0xA0..0xA7 => {self.and(b)}
					0xA8..0xAF => {self.xor(b)}
					0xB0..0xB7 => {self.or(b)}
					0xB8..0xBF => {self.cp(b)}
					_ => fail!("crash and burn : {:X}", n)
				}
			},

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
			0xE6 => {self.and(n); self.regs.pc.v += 1},
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
			0xD0 => {if !self.check_carry_flag() {self.ret()}}
			0xF0 => {
				let mut addr : u16 = 0xFF00;
				addr += n as u16;
				self.regs.af.set_high(self.mem.readbyte(addr));
				self.regs.pc.v += 1;
			},
			0xF3 => {println("WARNING: DI")},
			0xFA => {
				self.regs.pc.v += 2;
				self.regs.af.set_high(self.mem.readbyte(nn))
			},
			0xFE => {self.regs.pc.v += 1; self.cp(n)},
			_ => {fail!("Unimplemented OP: {:X}h", op)},
		}

		self.regs.pc.v += 1;
		self.run_clock();
	}
}

