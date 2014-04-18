
use std::io::println;
use std::str::from_utf8;
use std::io::stderr;

pub struct Mem {
	pub mbc_type : u8,
	pub mbc_rom_low : u8,
	pub mbc_ram_n : u8,
	pub mbc_romram : bool,
	pub mbc_ram_enable : bool,
	pub mem : [u8, ..0x10000],
	pub rom : ~[u8],
	buttons : bool,
	pub kup : bool,
	pub kdown : bool,
	pub kright : bool,
	pub kleft : bool,
	pub ka : bool,
	pub kb : bool,
	pub kselect : bool,
	pub kstart : bool,
}

impl Mem {
	pub fn new(rom : ~[u8]) -> Mem {
		let mut mem = [0, ..0x10000];
		mem[0xFF40] = 0x91;
		mem[0xFF47] = 0xFC;
		mem[0xFF48] = 0xFF;
		mem[0xFF49] = 0xFF;
		mem[0xFF4D] = 0xFF;
		mem[0xFFFF] = 0;
		let mbc_type = match rom[0x147] {
			0 => 0,
			0x1..0x3 => 1,
			0x5..0x6 => 2,
			0x8..0x9 => 0,
			0xB..0xD => fail!("Cart type not supported: 0x{:X} (mmm)", rom[0x147]),
			0xF..0x13 => 3,
			0x15..0x17 => 4,
			0x19..0x1E => 5,
			_ => fail!("Cart type not supported: 0x{:X} (weird thing)", rom[0x147]),
		};
		Mem {
			mbc_type : mbc_type,
			mbc_rom_low : 0,
			mbc_ram_n : 0,
			mbc_ram_enable : false,
			mbc_romram : false,
			mem : mem,
			rom : rom,
			buttons : false,
			kup : false,
			kdown : false,
			kright : false,
			kleft : false,
			ka : false,
			kb : false,
			kselect : false,
			kstart : false,
		}
	}
	pub fn rom_bank(&self) -> u16 {
		if self.mbc_type == 2 {
			fail!("TODO");
		} else if self.mbc_type == 1 {
			let mut n = if self.mbc_romram {
				self.mbc_rom_low | self.mbc_ram_n << 5
			} else {
				self.mbc_rom_low
			};
			if n == 0 {
				n = 1;
			}
			n as u16
		} else if self.mbc_type == 3 {
			self.mbc_rom_low as u16
		} else if self.mbc_type == 5 {
			if self.mbc_romram {
				self.mbc_rom_low as u16 | 1 << 9
			} else {
				self.mbc_rom_low as u16
			}
		} else {
			fail!("lel");
		}
	}
	pub fn ram_bank(&self) -> u8 {
		if self.mbc_type == 1 {
			if self.mbc_romram {
				0
			} else {
				self.mbc_ram_n
			}
		} else {
				self.mbc_ram_n
		}
	}
	pub fn readbyte(&self, offset : u16) -> u8 {
		if offset < 0x3FFF {
			self.rom[offset as uint ]
		} else if offset < 0x7FFF {
			if self.mbc_type > 0 {
				self.rom[(offset+0x4000*(self.rom_bank()-1)) as uint]
			} else {
				self.rom[offset as uint]
			}
		} else if offset == 0xFF00 {
			let a = if self.buttons {
				!self.ka as u8 |
				!self.kb as u8 << 1 |
				!self.kselect as u8 << 2 |
				!self.kstart as u8 << 3 |
				0x10
			} else {
				!self.kright as u8 |
				!self.kleft as u8 << 1 |
				!self.kup as u8 << 2 |
				!self.kdown as u8 << 3 |
				0x20
			};
			a | 0xC0
		} else {
			self.mem[offset as uint]
		}
	}
	pub fn writebyte(&mut self, offset : u16, value : u8) {
		let mtype = self.mbc_type;
		if offset < 0x2000 {
			if mtype == 1 || mtype == 5 {
				self.mbc_ram_enable = value == 0xA;
			}
			println("WARNING: wrote at < 0x2000");
		} else if offset < 0x4000 {
			if mtype == 5 {
				if offset < 0x3000 {
					self.mbc_rom_low = value;
				} else {
					self.mbc_romram = value != 0;
				}
			} else if mtype == 1 {
				self.mbc_rom_low = value & 0b11111;
				println!("rom bank: {:X}", self.rom_bank());
			} else if mtype == 3 {
				self.mbc_rom_low = value & 0b1111111;
				println!("rom bank: {:X}", self.rom_bank());
			} else {
				println!("WARNING: wrote at {:x}", offset);
			}
		} else if offset < 0x6000 {
			if mtype == 1 {
				self.mbc_ram_n = value & 0b11;
				println!("rom bank: {:X}", self.rom_bank());
			} else if mtype == 3 {
				self.mbc_romram = value != 0;
			} else if mtype == 5 {
				self.mbc_ram_n = value & 0xF;
			} else {
				println!("WARNING: wrote at {:x}", offset);
			}
		} else if offset < 0x7FFF {
			if mtype != 2 && mtype != 0 {
				self.mbc_romram = value & 1 == 1;
				println!("rom bank: {:X}", self.rom_bank());
			} else {
				println!("WARNING: wrote at {:x}", offset);
			}
		} else if offset == 0xFF00 {
			if value >> 4 & 1 == 0 {
				self.buttons = false
			} else if value >> 5 & 1 == 0 {
				self.buttons = true
			}
		} else if offset == 0xFF02 {
			if value == 0x81 {
				let c = ~[self.mem[0xFF01]];
				let cs = match from_utf8(c) {
					Some(g) => g,
					None => fail!("Couldn't decode game title")
				};
				let mut stde = stderr();
				stde.write_str(cs);
				self.request_interrupt(2);
			}
		} else {
			self.mem[offset as uint] = value;
		}
	}
	/*
	pub fn read(&self, offset : u16, len : u16) -> ~[u8] {
		let mut r = ~[];
		for i in range(0, len) {
			r.push(self.readbyte(offset+i));
		}
		return r;
	}
	*/
	pub fn write(&mut self, offset : u16, bytes : ~[u8]) {
		for i in range(0, bytes.len()) {
			let b = bytes[i];
			self.writebyte(offset+i as u16, b);
		}
	}
	pub fn read16(&self, offset : u16) -> u16 {
		self.readbyte(offset+1) as u16 << 8 | self.readbyte(offset) as u16
	}
	/*
	fn write_u16(&mut self, offset : u16, value : u16) {
		self.writebyte(offset as u16, (value & 0xFF) as u8);
		self.writebyte(offset+1 as u16, (value >> 8) as u8);
	}
	*/
	pub fn request_interrupt(&mut self, n : u8) {
		let f = self.readbyte(0xFF0F);
		self.writebyte(0xFF0F, f | 1 << n);
	}
}

