

use std::io::println;

pub struct Mem {
	pub bank_n : u16,
	pub mem : [u8, ..0x10000],
	pub rom : ~[u8],
}

impl Mem {
	pub fn readbyte(&self, offset : u16) -> u8 {
		if offset < 0x3FFF {
			self.rom[offset as uint ]
		} else if offset < 0x7FFF {
			self.rom[(offset+0x4000*self.bank_n) as uint]
		} else {
			self.mem[offset as uint]
		}
	}
	pub fn writebyte(&mut self, offset : u16, value : u8) {
		if offset < 0x3FFF {
			println("WARNING: wrote at < 0x3FFF");
		} else if offset < 0x7FFF {
			println("WARNING: wrote at < 0x7FFF");
		} else {
			self.mem[offset as uint] = value;
		}
	}
	pub fn read(&self, offset : u16, len : u16) -> ~[u8] {
		let mut r = ~[];
		for i in range(0, len) {
			r.push(self.readbyte(offset+i));
		}
		return r;
	}
	pub fn write(&mut self, offset : u16, bytes : ~[u8]) {
		for i in range(0, bytes.len()) {
			let b = bytes[i];
			self.writebyte(offset+i as u16, b);
		}
	}
	pub fn read16(&self, offset : u16) -> u16 {
		self.readbyte(offset+1) as u16 << 8 | self.readbyte(offset) as u16
	}
	fn write_u16(&mut self, offset : u16, value : u16) {
		self.writebyte(offset as u16, (value & 0xFF) as u8);
		self.writebyte(offset+1 as u16, (value >> 8) as u8);
	}
}

