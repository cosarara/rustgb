
use mem::Mem;
struct Reg {
	v : u16
}

impl Reg {
	fn high(~self) -> HReg {
		HReg { parent : self , h : true }
	}
	fn low(~self) -> HReg {
		HReg { parent : self , h : false }
	}
}

struct HReg {
	parent : ~Reg,
	h : bool,
}

impl HReg {
	fn set(&mut self, v : u8) {
		// Is the high half
		if self.h {
			self.parent.v = (v << 8) as u16 | (self.parent.v & 0xFF) as u16
		} else {
			self.parent.v = (self.parent.v & 0xFF00) as u16 | v as u16
		}
	}
	fn get(&self) -> u8 {
		if self.h {
			(self.parent.v & 0xFF00) as u8
		} else {
			(self.parent.v & 0xFF) as u8
		}
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
			mem : Mem { bank_n : 1, mem : [0, ..0xFFFF], rom : rom },
		}
	}
	pub fn run(&mut self) {

	}
}

