

pub struct Mem {
	pub bank_n : uint,
	pub mem : [u8, ..0xFFFF],
	pub rom : ~[u8],
}

impl Mem {
	pub fn readbyte(&self, offset : uint) -> u8 {
		if offset < 0x3FFF {
			self.rom[offset]
		} else if offset < 0x7FFF {
			self.rom[offset+0x4000*self.bank_n]
		} else {
			self.mem[offset]
		}
	}
	pub fn writebyte(&mut self, offset : uint, value : u8) {
		if offset < 0x3FFF {
			fail!("TODO");
		} else if offset < 0x7FFF {
			fail!("TODO");
		} else {
			self.mem[offset] = value;
		}
	}
	fn read(&self, offset : uint, len : uint) -> ~[u8] {
		let mut r = ~[];
		for i in range(0, len) {
			r.push(self.readbyte(offset+i));
		}
		return r;
	}
	fn write(&mut self, offset : uint, bytes : ~[u8]) {
		for i in range(0, bytes.len()) {
			let b = bytes[i];
			self.writebyte(offset+i, b);
		}
	}
}

