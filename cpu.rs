

// This, like any cpu emulator, is a fucking mess.
extern crate std;
use mem::Mem;
struct Reg {
	pub v : u16
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
	fn add_high(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o+v);
		(o, o+v)
	}
	fn add(&mut self, v : u16) -> (u16, u16) {
		let o = self.v;
		self.v = o+v;
		(o, o+v)
	}
	fn addi8(&mut self, v : i8) -> (u16, u16) {
		let o = self.v;
		let r = (o as i16 + v as i16) as u16;
		self.v = r;
		(o, r)
	}
	fn sub_high(&mut self, v : u8) -> (u8, u8) {
		let o = self.get_high();
		self.set_high(o-v);
		(o, o-v)
	}

	fn to_bytes(&self) -> [u8, ..2] {
		[self.get_low(), self.get_high()]
	}
}

struct Regs {
	pub af : Reg,
	pub bc : Reg,
	pub de : Reg,
	pub hl : Reg,
	pub sp : Reg,
	pub pc : Reg
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

pub struct Cpu<'rom> {
	pub regs : Regs,
	pub mem : Mem<'rom>,
	clock : uint,
	screen_mode : int,
	pub drawing : bool,
	interrupts_enabled : bool,
	last_op : u8,
	halted : bool,
	log : bool,
}

impl<'rom> Cpu<'rom> {
	pub fn new<'a>(rom : &'a [u8]) -> Cpu<'a> {
		Cpu {
			regs : Regs::new(),
			mem : Mem::new(rom),
			clock : 0,
			screen_mode : 0,
			drawing : false,
			interrupts_enabled : false,
			last_op : 0,
			halted : false,
			log : std::os::args().len() > 2,
		}
	}
	fn ei(&mut self) {
		self.mem.ime_delay = 2;
	}
	fn di(&mut self) {
		self.interrupts_enabled = false;
	}
	// Tests for carry
	fn ca8(&mut self, (old, new) : (u8, u8)) -> u8 {
		self.set_carry_flag(new < old);
		new
	}
	// subtraction
	fn cs8(&mut self, (old, new) : (u8, u8)) -> u8 {
		self.set_carry_flag(new > old);
		new
	}
	// Tests for zero in 8bit registers
	fn z8(&mut self, val : u8) {
		self.set_zero_flag(val == 0);
	}

	fn incflags(&mut self, (_, r) : (u8, u8)) {
		self.z8(r);
		self.set_addsub_flag(false);
		self.set_hc_flag((r&0xF) == 0);
	}

	fn addflags(&mut self, t : (u8, u8)) {
		let (a, f) = t;
		self.ca8(t);
		self.incflags(t);
		self.set_hc_flag(((f&0xF) < (a&0xF)));
	}

	fn addflags16(&mut self, t : (u16, u16)) {
		self.set_addsub_flag(false);
		let (a, f) = t;
		self.set_hc_flag((f&0xFFF) < (a&0xFFF));
	}

	fn decflags(&mut self, (_, r) : (u8, u8)) {
		self.z8(r);
		self.set_addsub_flag(true);
		self.set_hc_flag((r&0xF) == 0xF);
	}

	fn subflags(&mut self, t : (u8, u8)) {
		self.cs8(t);
		self.decflags(t);
		let (a, f) = t;
		self.set_hc_flag(((a&0xF) < (f&0xF)));
	}

	fn adc(&mut self, b : u8) {
		let c = self.check_carry_flag() as u8;
		let a = self.regs.af.get_high();
		let f = self.regs.af.add_high(b+c);
		self.addflags(f);
		let adc : uint = a as uint + b as uint + c as uint;
		self.set_carry_flag(adc > 0xFF);
		let h = (a & 0xF) + (b & 0xF) + c > 0xF;
		self.set_hc_flag(h);
	}

	fn sbc(&mut self, b : u8) {
		let c = self.check_carry_flag();
		let a = self.regs.af.get_high();
		let f = self.regs.af.sub_high(b+c as u8);
		self.subflags(f);
		let h : bool = ((a & 0xF) as int - (b & 0xF) as int - c as int) < 0 as int;
		let sbc : int = a as int - b as int - c as int;
		self.set_carry_flag(sbc < 0);
		self.set_hc_flag(h);
	}
	fn rlc(&mut self, x : u8) -> u8 {
		let a = (x << 1) | (x >> 7);
		self.set_carry_flag(a & 1 == 1);
		self.set_zero_flag(a == 0);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		a
	}
	fn rrc(&mut self, x : u8) -> u8 {
		let a = (x >> 1) | (x << 7);
		self.set_carry_flag(a >> 7 == 1);
		self.set_zero_flag(a == 0);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		a
	}
	fn rl(&mut self, x : u8) -> u8 {
		let b = x >> 7;
		let r = x << 1 | (if self.check_carry_flag() {1} else {0});
		self.set_carry_flag(b == 1);
		self.set_zero_flag(r == 0);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		r
	}
	fn rr(&mut self, x : u8) -> u8 {
		let b = x & 1;
		let r = x >> 1 | (if self.check_carry_flag() {1} else {0}) << 7;
		self.set_carry_flag(b == 1);
		self.set_zero_flag(r == 0);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		r
	}
	fn push(&mut self, v : u16) {
		self.regs.sp.v -= 2;
		let m : [u8, ..2] = [(v & 0xFF) as u8, (v >> 8) as u8];
		self.mem.write(self.regs.sp.v, m);
	}
	fn pop(&mut self) -> u16 {
		let mut r = self.mem.readbyte(self.regs.sp.v) as u16;
		self.regs.sp.v += 1;
		r |= self.mem.readbyte(self.regs.sp.v) as u16 << 8;
		self.regs.sp.v += 1;
		r
	}
	fn call(&mut self, v : u16) {
		let a = self.regs.pc.v+1;
		self.push(a);
		self.regs.pc.v = v-1;
	}
	fn call_interrupt(&mut self, v : u16) {
		let r = self.regs.pc.v + if self.halted {1} else {0};
		self.push(r);
		self.regs.pc.v = v;
	}
	fn ret(&mut self) {
		self.regs.pc.v = self.pop()-1;
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
	fn check_addsub_flag(&mut self) -> bool {
		self.regs.af.get_low() & (1 << 6) != 0
	}
	fn set_hc_flag(&mut self, v : bool) {
		let n = if v {
			self.regs.af.get_low() | (1 << 5)
		} else {
			self.regs.af.get_low() & !(1 << 5)
		};
		self.regs.af.set_low(n);
	}
	fn check_hc_flag(&mut self) -> bool {
		self.regs.af.get_low() & (1 << 5) != 0
	}
	fn and(&mut self, v : u8) {
		let a = self.regs.af.get_high() & v;
		self.set_zero_flag(a == 0);
		self.set_carry_flag(false);
		self.set_addsub_flag(false);
		self.set_hc_flag(true);
		self.regs.af.set_high(a)
	}
	fn or(&mut self, v : u8) {
		let a = self.regs.af.get_high() | v;
		self.set_zero_flag(a == 0);
		self.set_carry_flag(false);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		self.regs.af.set_high(a)
	}
	fn xor(&mut self, v : u8) {
		let a = self.regs.af.get_high() ^ v;
		self.set_zero_flag(a == 0);
		self.set_carry_flag(false);
		self.set_addsub_flag(false);
		self.set_hc_flag(false);
		self.regs.af.set_high(a)
	}
	fn cp(&mut self, v : u8) {
		let a = self.regs.af.get_high();
		self.set_zero_flag(a == v);
		self.set_carry_flag(a < v);
		self.set_addsub_flag(true);
		self.set_hc_flag(a&0xf < (a-v)&0xf);
	}
	fn jr(&mut self, v : u8) {
		self.regs.pc.v = (self.regs.pc.v as i16 + (v as i8) as i16) as u16 + 1;
	}
	pub fn run_clock(&mut self) {
		self.clock += 4; // TODO: precise cycles
		// self.mem.mem[0xff44] holds the line number
		match self.screen_mode {
			// HBlank
			0 => {
				if self.clock >= 204 {
					self.clock = 0;
					self.mem.mem[0xff44] += 1;
					if self.mem.mem[0xff44] == 143 {
						self.drawing = true;
						self.screen_mode = 1; // Finish, go to VBlank
						self.mem.request_interrupt(0);
					} else {
						self.screen_mode = 2;
					}
				}
			},
			// VBlank
			1 => {
				if self.clock >= 456 {
					self.clock = 0;
					self.mem.mem[0xff44] += 1;
					if self.mem.mem[0xff44] > 153 {
						self.screen_mode = 2;
						self.mem.mem[0xff44] = 0;
					}
				}
			},
			// OAM Read
			2 => {
				if self.clock >= 80 {
					self.clock = 0;
					self.screen_mode = 3;
				}
			},
			// VRAM Read
			3 => {
				if self.clock >= 172 {
					self.clock = 0;
					self.screen_mode = 0;
				}
			},
			_ => fail!("Wat"),
		}
		let tcontrol = self.mem.readbyte(0xFF07);
		//println!("clock ctrl : {:X}", tcontrol);
		let tspeed = tcontrol & 3;
		let tclock = self.clock / 4;
		if tcontrol & 4 != 0 {
			//println("clocking");
			if tclock % 16 == 0 {
				let div = self.mem.readbyte(0xFF04);
				self.mem.writebyte(0xFF04, div + 1);
			}
			let a = match tspeed {
				0 => 64,
				1 => 1,
				2 => 4,
				3 => 16,
				_ => fail!("Do you even binary?")
			};
			let mut count = self.mem.readbyte(0xFF05);
			if tclock % a == 0 {
				count += 1;
				if count == 0 {
					//println("int 50");
					self.mem.request_interrupt(2);
					count = self.mem.readbyte(0xFF06);
				}
				self.mem.writebyte(0xFF05, count);
			}
		}
	}
	pub fn next(&mut self) {
		//if self.regs.pc.v == 0x03C6 {
		//	let mut file = File::create(&Path::new("ram_dump.bin"));
		//	file.write(self.mem.mem);
		//	fail!("quit")
		//}
		let op : u8 = self.mem.readbyte(self.regs.pc.v);
		let n : u8 = self.mem.readbyte(self.regs.pc.v+1);
		let nn : u16 = n as u16 | self.mem.readbyte(self.regs.pc.v+2) as u16 << 8;
		if self.log && !(op == 0x76 && self.last_op == 0x76) {
			println!("PC: {:04X} | OPCODE: {:02X} | MEM: {:02X}{:02X}",
				self.regs.pc.v, op, n, nn>>8);
			/*println!("{:04X} {:02X} {:02X} {:02X}\t\tSP: {:04X} AF: {:04X} BC: {:04X} DE: {:04X} HL: {:04X} On Stack: {:04X}",
					 self.regs.pc.v, op, n, nn>>8, self.regs.sp.v,
					 self.regs.af.v, self.regs.bc.v, self.regs.de.v, self.regs.hl.v,
					 self.mem.read16(self.regs.sp.v));
					 */
			/*println!("-6 {:04X} -4 {:04X} -2 {:04X} +0 {:04X} +2 {:04X} +4 {:04X}",
					 self.mem.read16(self.regs.hl.v-6),
					 self.mem.read16(self.regs.hl.v-4),
					 self.mem.read16(self.regs.hl.v-2),
					 self.mem.read16(self.regs.hl.v),
					 self.mem.read16(self.regs.hl.v+2),
					 self.mem.read16(self.regs.hl.v+4));*/
		}
		// Immutable copies
		let hl = self.regs.hl.v;
		let bc = self.regs.bc.v;
		let de = self.regs.de.v;
		let af = self.regs.af.v;
		match op {
			0x00 => {},
			0x01 => {self.regs.bc.v = nn; self.regs.pc.v += 2},
			0x02 => {self.mem.writebyte(self.regs.bc.v, self.regs.af.get_high())},
			0x03 => {self.regs.bc.v += 1},
			0x04 => {let a = self.regs.bc.inc_high(); self.incflags(a)},
			0x05 => {let a = self.regs.bc.dec_high(); self.decflags(a)},
			0x06 => {self.regs.bc.set_high(n); self.regs.pc.v += 1},
			0x07 => {
				let a = self.regs.af.get_high();
				let b = self.rlc(a);
				self.regs.af.set_high(b);
				self.set_zero_flag(false);
			},
			0x08 => {
				self.mem.write(nn, self.regs.sp.to_bytes());
				self.regs.pc.v += 2},
			0x09 => {let r = self.regs.hl.add(self.regs.bc.v); self.addflags16(r)},
			0x0A => {self.regs.af.set_high(self.mem.readbyte(self.regs.bc.v))},
			0x0B => {self.regs.bc.v -= 1},
			0x0C => {let a = self.regs.bc.inc_low(); self.incflags(a)},
			0x0D => {let a = self.regs.bc.dec_low(); self.decflags(a)},
			0x0E => {self.regs.bc.set_low(n); self.regs.pc.v += 1},
			0x0F => {
				let a = self.regs.af.get_high();
				let b = self.rrc(a);
				self.regs.af.set_high(b);
				self.set_zero_flag(false);
			},
			0x10 => {fail!("STOP")},
			0x11 => {self.regs.de.v = nn; self.regs.pc.v += 2},
			0x12 => {self.mem.writebyte(self.regs.de.v, self.regs.af.get_high())},
			0x13 => {self.regs.de.v += 1},
			0x14 => {let a = self.regs.de.inc_high(); self.incflags(a)},
			0x15 => {let a = self.regs.de.dec_high(); self.decflags(a)},
			0x16 => {self.regs.de.set_high(n); self.regs.pc.v += 1},
			0x17 => {
				let a = self.regs.af.get_high();
				let b = self.rl(a);
				self.regs.af.set_high(b);
				self.set_zero_flag(false);
			},
			0x18 => self.jr(n),
			0x19 => {let r = self.regs.hl.add(self.regs.de.v); self.addflags16(r)},
			0x1A => {self.regs.af.set_high(self.mem.readbyte(self.regs.de.v))},
			0x1B => {self.regs.de.v -= 1},
			0x1C => {let a = self.regs.de.inc_low(); self.incflags(a)},
			0x1D => {let a = self.regs.de.dec_low(); self.decflags(a)},
			0x1E => {self.regs.de.set_low(n); self.regs.pc.v += 1},
			0x1F => {
				let a = self.regs.af.get_high();
				let b = self.rr(a);
				self.regs.af.set_high(b);
				self.set_zero_flag(false);
			},
			0x20 => if !self.check_zero_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x21 => {self.regs.pc.v += 2; self.regs.hl.v = nn},
			0x22 => {
				let addr = self.regs.hl.v;
				self.mem.writebyte(addr, self.regs.af.get_high());
				self.regs.hl.v += 1},
			0x23 => {self.regs.hl.v += 1},
			0x24 => {let a = self.regs.hl.inc_high(); self.incflags(a)},
			0x25 => {let a = self.regs.hl.dec_high(); self.decflags(a)},
			0x26 => {self.regs.hl.set_high(n); self.regs.pc.v += 1},
			0x27 => { //DAA
				let mut a = self.regs.af.get_high();
				if !self.check_addsub_flag() {
					if a > 0x99 || self.check_carry_flag() {
						a += 0x60;
						self.set_carry_flag(true);
					}
					if a & 0xF > 0x9 || self.check_hc_flag() {
						a += 0x6;
						self.set_hc_flag(false);
					}
				} else if self.check_carry_flag() && self.check_hc_flag() {
					a += 0x9A;
					self.set_hc_flag(false);
				} else if self.check_carry_flag() {
					a += 0xA0;
				} else if self.check_hc_flag() {
					a += 0xFA;
					self.set_hc_flag(false);
				}
				self.regs.af.set_high(a);
				self.set_zero_flag(a == 0);
			}
			0x28 => if self.check_zero_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x29 => {let r = self.regs.hl.add(hl); self.addflags16(r)},
			0x2A => {
				let addr = self.regs.hl.v;
				self.regs.af.set_high(self.mem.readbyte(addr));
				self.regs.hl.v += 1},
			0x2B => {self.regs.hl.v -= 1},
			0x2C => {let a = self.regs.hl.inc_low(); self.incflags(a)},
			0x2D => {let a = self.regs.hl.dec_low(); self.decflags(a)},
			0x2E => {self.regs.hl.set_low(n); self.regs.pc.v += 1},
			0x2F => {let a = self.regs.af.get_high(); self.regs.af.set_high(a^0xFF);
				self.set_addsub_flag(true); self.set_hc_flag(true)},

			0x30 => if !self.check_carry_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x31 => {self.regs.sp.v = nn; self.regs.pc.v += 2},
			0x32 => {
				let addr = self.regs.hl.v;
				self.mem.writebyte(addr, self.regs.af.get_high());
				self.regs.hl.v -= 1},
			0x33 => {self.regs.sp.v += 1},
			0x34 => {
				let addr = self.regs.hl.v;
				let a = self.mem.readbyte(addr);
				self.mem.writebyte(addr, a+1);
				self.incflags((a, a+1))},
			0x35 => {
				let addr = self.regs.hl.v;
				let a = self.mem.readbyte(addr);
				self.mem.writebyte(addr, a-1);
				self.decflags((a, a-1))},
			0x36 => {let addr = self.regs.hl.v;
				self.mem.writebyte(addr, n);
				self.regs.pc.v += 1},
			0x37 => {
				self.set_carry_flag(true);
				self.set_addsub_flag(false);
				self.set_hc_flag(false);
			},
			0x38 => if self.check_carry_flag() {self.jr(n)} else {self.regs.pc.v += 1},
			0x39 => {let r = self.regs.hl.add(self.regs.sp.v); self.addflags16(r)},
			0x3A => {
				let addr = self.regs.hl.v;
				self.regs.af.set_high(self.mem.readbyte(addr));
				self.regs.hl.v -= 1},
			0x3B => {self.regs.sp.v -= 1},
			0x3C => {let a = self.regs.af.inc_high(); self.incflags(a)},
			0x3D => {let a = self.regs.af.dec_high(); self.decflags(a)},
			0x3E => {self.regs.af.set_high(n); self.regs.pc.v += 1},
			0x3F => {
				let c = self.check_carry_flag();
				self.set_carry_flag(!c);
				self.set_addsub_flag(false);
				self.set_hc_flag(false);
			},
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
					0x70..0x77 => if op == 0x76 {
						self.halted = true;
						self.regs.pc.v -= 1; // Just wait here ok?
						//if !self.interrupts_enabled || self.mem.readbyte(0xFFFF) == 0 {
						//	fail!("Halt with interrupts disabled, I don't know what to do.");
						//}
						//println!("ie {:X}", self.mem.readbyte(0xFFFF));
						//println!("tc {:X}", self.mem.readbyte(0xFF07));
					} else {
						self.mem.writebyte(self.regs.hl.v, b)
					},
					0x78..0x7F => self.regs.af.set_high(b),
					0x80..0x87 => {
						let f = self.regs.af.add_high(b);
						self.addflags(f)},
					0x88..0x8F => self.adc(b), //ADC
					0x90..0x97 => {
						let f = self.regs.af.sub_high(b);
						self.subflags(f)},
					0x98..0x9F => self.sbc(b), //SBC
					0xA0..0xA7 => {self.and(b)}
					0xA8..0xAF => {self.xor(b)}
					0xB0..0xB7 => {self.or(b)}
					0xB8..0xBF => {self.cp(b)}
					_ => fail!("crash and burn : {:X}", op)
				}
			},

			0xC0 => {if !self.check_zero_flag() {self.ret()}},
			0xC1 => {self.regs.bc.v = self.pop()},
			0xC2 => if !self.check_zero_flag() {self.regs.pc.v = nn-1} else {self.regs.pc.v += 2},
			0xC3 => {self.regs.pc.v = nn-1},
			0xC4 => {self.regs.pc.v += 2; if !self.check_zero_flag() {self.call(nn)}},
			0xC5 => {self.push(bc)},
			0xC6 => {
				let f = self.regs.af.add_high(n);
				self.addflags(f);
				self.regs.pc.v += 1},
			0xC7 => self.call(0),
			0xC8 => {if self.check_zero_flag() {self.ret()}},
			0xC9 => {self.ret()},
			0xCA => if self.check_zero_flag() {self.regs.pc.v = nn-1} else {self.regs.pc.v += 2},
			0xCB => {
				fn f(s : &mut Cpu, n: u8, x: u8) -> u8 {
					if n < 0x8 { // RLC
						s.rlc(x)
					} else if n < 0x10 { // RRC
						s.rrc(x)
					} else if n < 0x18 { // RL
						s.rl(x)
					} else if n < 0x20 { // RR
						s.rr(x)
					} else if n < 0x28 { // SLA
						s.set_addsub_flag(false);
						s.set_hc_flag(false);
						s.set_carry_flag(x >> 7 == 1);
						let r = x << 1;
						s.set_zero_flag(r == 0);
						r
					} else if n < 0x30 { // SRA
						s.set_addsub_flag(false);
						s.set_hc_flag(false);
						s.set_carry_flag(x & 1 == 1);
						let r = (x & 0x80) | (x >> 1);
						s.set_zero_flag(r == 0);
						r
					} else if n < 0x38 { // SWAP
						s.set_carry_flag(false);
						s.set_addsub_flag(false);
						s.set_hc_flag(false);
						let r = x << 4 | x >> 4;
						s.set_zero_flag(r == 0);
						r
					} else if n < 0x40 { // SRL
						s.set_carry_flag(x & 1 == 1);
						s.set_addsub_flag(false);
						s.set_hc_flag(false);
						let r = x >> 1;
						s.set_zero_flag(r == 0);
						r
					} else if n < 0x80 { // BIT
						let b = n >> 3 & 7;
						let c = (x >> b as uint) & 1;
						s.set_zero_flag(c != 1);
						s.set_addsub_flag(false);
						s.set_hc_flag(true);
						x
					} else if n < 0xC0 { // RES
						let b = (n >> 3) & 0x7;
						x & (0xFF ^ (1 << b as uint))
					} else { // SET
						let b = (n >> 3) & 0x7;
						x | (1 << b as uint)
					}
				}
				if n < 0x40 || n > 0x80 {
					match n & 7 {
						0 => {let x = self.regs.bc.get_high();
							let r = f(self, n, x); self.regs.bc.set_high(r)},
						1 => {let x = self.regs.bc.get_low();
							let r = f(self, n, x); self.regs.bc.set_low(r)},
						2 => {let x = self.regs.de.get_high(); 
							let r = f(self, n, x); self.regs.de.set_high(r)},
						3 => {let x = self.regs.de.get_low(); 
							let r = f(self, n, x); self.regs.de.set_low(r)},
						4 => {let x = self.regs.hl.get_high(); 
							let r = f(self, n, x); self.regs.hl.set_high(r)},
						5 => {let x = self.regs.hl.get_low(); 
							let r = f(self, n, x); self.regs.hl.set_low(r)},
						6 => {let a = self.regs.hl.v;
							let x = self.mem.readbyte(a);
							let r = f(self, n, x);
							self.mem.writebyte(a, r)},
						7 => {let x = self.regs.af.get_high();
							let r = f(self, n, x); self.regs.af.set_high(r)},
						_ => fail!("wat.")
					}
				} else {
					match n & 7 {
						0 => {let x = self.regs.bc.get_high(); f(self, n, x);},
						1 => {let x = self.regs.bc.get_low(); f(self, n, x);},
						2 => {let x = self.regs.de.get_high(); f(self, n, x);},
						3 => {let x = self.regs.de.get_low(); f(self, n, x);},
						4 => {let x = self.regs.hl.get_high(); f(self, n, x);},
						5 => {let x = self.regs.hl.get_low(); f(self, n, x);},
						6 => {let a = self.regs.hl.v;
							let x = self.mem.readbyte(a);
							f(self, n, x);},
						7 => {let x = self.regs.af.get_high(); f(self, n, x);},
						_ => fail!("wat.")
					}
				}
				self.regs.pc.v += 1;
			},
			0xCC => {self.regs.pc.v += 2; if self.check_zero_flag() {self.call(nn)}},
			0xCD => {self.regs.pc.v += 2; self.call(nn)},
			0xCE => {
				self.adc(n);
				self.regs.pc.v += 1},
			0xCF => self.call(0x08),
			0xD0 => {if !self.check_carry_flag() {self.ret()}},
			0xD1 => {self.regs.de.v = self.pop()},
			0xD2 => if !self.check_carry_flag() {self.regs.pc.v = nn-1} else {self.regs.pc.v += 2},
			// D3 does not exist
			0xD4 => {self.regs.pc.v += 2; if !self.check_carry_flag() {self.call(nn)}},
			0xD5 => {self.push(de)},
			0xD6 => {
				let f = self.regs.af.sub_high(n);
				self.subflags(f);
				self.regs.pc.v += 1},
			0xD7 => self.call(0x10),
			0xD8 => {if self.check_carry_flag() {self.ret()}},
			0xD9 => {self.ei(); self.ret()},
			0xDA => if self.check_carry_flag() {self.regs.pc.v = nn-1} else {self.regs.pc.v += 2},
			// DB does not exist
			0xDC => {self.regs.pc.v += 2; if self.check_carry_flag() {self.call(nn)}},
			// DD does not exist
			0xDE => {
				self.regs.pc.v += 1;
				self.sbc(n)},
			0xDF => self.call(0x18),
			0xE0 => {
				let addr : u16 = 0xFF00 + n as u16;
				self.mem.writebyte(addr, self.regs.af.get_high());
				self.regs.pc.v += 1;
			},
			0xE1 => {self.regs.hl.v = self.pop()},
			0xE2 => {
				let addr : u16 = 0xFF00 + self.regs.bc.get_low() as u16;
				self.mem.writebyte(addr, self.regs.af.get_high());
			},
			// E3 and E4 do not exist
			0xE5 => {self.push(hl)},
			0xE6 => {self.and(n); self.regs.pc.v += 1},
			0xE7 => self.call(0x20),
			0xE8 => {
				let sp1 = self.regs.sp.v;
				let sn = n as i8;
				self.regs.sp.addi8(sn);
				//self.addflags16(r);
				let sp2 = self.regs.sp.v;
				// I have no idea of what I'm doing here,
				// looked it up from Gameboy-Online source
				let f = sp1 as i16^sn as i16^sp2 as i16;
				self.set_hc_flag(f&0x10 == 0x10);
				self.set_carry_flag(f&0x100 == 0x100);
				self.set_zero_flag(false);
				self.set_addsub_flag(false);
				self.regs.pc.v += 1;},
			0xE9 => { // Docs says its a jump to (hl), but seems it's jp hl
				self.regs.pc.v = self.regs.hl.v-1},
			0xEA => {
				self.mem.writebyte(nn, self.regs.af.get_high());
				self.regs.pc.v += 2;
			},
			// EB, EC and ED do not exist
			0xEE => {self.xor(n); self.regs.pc.v += 1},
			0xEF => self.call(0x28),
			0xF0 => {
				let addr : u16 = 0xFF00 + n as u16;
				self.regs.af.set_high(self.mem.readbyte(addr));
				self.regs.pc.v += 1;
			},
			0xF1 => {
				self.regs.af.v = self.pop();
				self.regs.af.v &= 0xFFF0;
			},
			0xF2 => {
				let addr : u16 = 0xFF00 + self.regs.bc.get_low() as u16;
				self.regs.af.set_high(self.mem.readbyte(addr));
			},
			0xF3 => {self.di()},
			// F4 does not exist
			0xF5 => {self.push(af)},
			0xF6 => {self.or(n); self.regs.pc.v += 1},
			0xF7 => self.call(0x30),
			0xF8 => {
				let sp = self.regs.sp.v;
				let sn = n as i8;
				let r = (sp as i16 + sn as i16) as u16;
				// Also don't really know what I'm doing here
				let f = sp as i16^sn as i16^r as i16;
				self.set_zero_flag(false);
				self.set_addsub_flag(false);
				self.set_hc_flag(f&0x10 == 0x10);
				self.set_carry_flag(f&0x100 == 0x100);
				self.regs.hl.v = r;
				self.regs.pc.v += 1;},
			0xF9 => {self.regs.sp.v = self.regs.hl.v},
			0xFA => {
				self.regs.pc.v += 2;
				self.regs.af.set_high(self.mem.readbyte(nn))
			},
			0xFB => self.ei(),
			// FC and FD do not exist
			0xFE => {self.regs.pc.v += 1; self.cp(n)},
			0xFF => self.call(0x38),
			_ => {fail!("Unimplemented OP: {:X}h", op)},
		}

		self.regs.pc.v += 1;
		self.last_op = op;
	}
	pub fn interrupts(&mut self) {
		self.mem.ime_delay = match self.mem.ime_delay {
			0 => 0,
			1 => {
				self.interrupts_enabled = true;
				0
			},
			2 => 1,
			_ => fail!("Unexpected IME delay")};
		for n in range(0, 4) {
			if !self.interrupts_enabled && !self.halted {
				return;
			}
			let a = match n {
				0 => 0x40,
				1 => 0x48,
				2 => 0x50,
				3 => 0x58,
				4 => 0x60,
				_ => fail!("Interrupt codes go from 0 to 4"),
			};
			let e = self.mem.readbyte(0xFFFF);
			let f = self.mem.readbyte(0xFF0F);
			if (f & e) >> n & 1 != 1 {
				continue;
			}
			//println!("Calling int {:X}h", a);
			if !self.halted {
				self.mem.writebyte(0xFF0F, f ^ 1 << n);
				self.call_interrupt(a);
			} else {
				self.halted = false;
				self.regs.pc.v += 1;
			}
			break;
		}
	}
}

