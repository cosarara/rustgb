
use std::io::println;
use std::str::from_utf8;
use std::io::stderr;

pub struct Mem {
	pub bank_n : u16,
	pub mbc_type : int,
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
		Mem {
			bank_n : 1,
			mbc_type : 0,
			mem : [0, ..0x10000],
			rom : rom,
			buttons : false,
			kup : false,
			kdown : false,
			kright : false,
			kleft : false,
			ka : false,
			kb : false,
			kselect : false,
			kstart : false
		}
	}
	pub fn readbyte(&self, offset : u16) -> u8 {
		if offset < 0x3FFF {
			self.rom[offset as uint ]
		} else if offset < 0x7FFF {
			if self.mbc_type > 0 {
				self.rom[(offset+0x4000*self.bank_n) as uint]
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
			//println!("eeeeee {:X}", a);
			a | 0xC0
		} else {
			self.mem[offset as uint]
		}
	}
	pub fn writebyte(&mut self, offset : u16, value : u8) {
		if offset < 0x3FFF {
			println("WARNING: wrote at < 0x3FFF");
		} else if offset < 0x7FFF {
			println("WARNING: wrote at < 0x7FFF");
		} else if offset == 0xFF00 {
			//println!("lalala {:X}", value);
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
}

