
extern crate sdl;
extern crate libc;
use sdl::video::Surface;
use sdl::video::Color;
use sdl::video::RGB;
use std::io::File;
use std::io::{File, Open, ReadWrite};
use cpu::Cpu;
use std::io::println;
use std::num;
use std::io::BufferedReader;
use libc::{c_int, c_void, time_t};
use std::ptr::null;
mod cpu;
mod mem;
mod events;

struct timeval {
	tv_sec: time_t,
	tv_usec: u32,
}

extern {
	fn gettimeofday(tp: *mut timeval, tzp: *c_void) -> c_int;
}

pub fn current_time_millis() -> u64 {
	unsafe {
		let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
		gettimeofday(&mut tv, null());
		(tv.tv_sec as u64) * 1000 + (tv.tv_usec as u64) / 1000
	}
}



fn draw(screen : &Surface, vram : &[u8], lcdc : u8) {
	fn putpixel(screen : &Surface, x : i16, y : i16, color : Color) {
		// Stupid implementation, but I don't want to fight C and unsafety now
		screen.fill_rect(Some(sdl::Rect {
                x: x,
                y: y,
                w: 1,
                h: 1
		}), color);
	}
	let t1 = match Surface::new(&[], 512, 512, 32, 0x000000ff, 0x0000ff00, 0x00ff0000, 0xff000000) {
        Ok(s) => s,
        Err(err) => fail!("failed to set video mode: {}", err)
	};
	let cols = 16;
	let base_tiledata_addr = if lcdc >> 4 & 1 == 0 { 0x800 } else { 0 };
	for tile in range(0, 255) {
		let taddr = tile * 16 + base_tiledata_addr;
		for line in range(0, 8) {
			let laddr = (taddr + 2*line) as uint;
			for pixel in range(0, 8) {
				let c = vram[laddr] >> 7 - pixel & 1 |
						(vram[laddr+1] >> 7 - pixel & 1) << 1;
				//println!("qwer {:X} {} {}", laddr, vram[laddr], vram[laddr+1]);
				putpixel(t1, (tile%cols*8+pixel) as i16, (tile/cols*8+line) as i16, match c {
					3 => RGB(0, 0, 0),
					2 => RGB(0x55, 0x55, 0x55),
					1 => RGB(0xAA, 0xAA, 0xAA),
					0 => RGB(0xFF, 0xFF, 0xFF),
					_ => fail!("you are terminated")
				});
			}
		}
	}
	let base_bgmap_addr = if lcdc >> 3 & 1 == 0 { 0x1800 } else { 0x1C00 };
	for row in range(0, 32) {
		for cell_n in range(0, 32) {
			let addr = (base_bgmap_addr+row*32+cell_n) as uint;
			let mut tile_n = vram[addr] as i16;
			if lcdc >> 4 & 1 == 0 {
				tile_n = cpu::sign(tile_n as u8) as i16;
				tile_n += 128;
			}
			let sx = tile_n%(cols as i16)*8;
			let sy = tile_n/(cols as i16)*8;
			screen.blit_rect(t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
				Some(sdl::Rect {x:(cell_n*8) as i16, y:(row*8) as i16, w:8, h:8}));
		}
	}
	screen.blit_at(t1, 0, 260);
}

#[test]
fn test_instr() {
	let mut f = BufferedReader::new(File::open(&Path::new("out")));
	let lines: ~[~str] = f.lines().map(|x| x.unwrap()).collect();

	let mut rom = ~[0 as u8, ..0x200];
	rom[0x100] = 0x27;
	let mut cpu = Cpu::new(rom);
	for i_ in range(0, 0xFFFF) {
		let i = i_ as u16;
		cpu.regs.af.v = i & 0xFFF0;
		cpu.regs.bc.v = 0;
		//cpu.regs.af.v = (i << 8 | 1 << 4);
		//cpu.regs.bc.v = (i & 0xFF00);
		println!("input: {:04X}, {:04X}", cpu.regs.af.v, cpu.regs.bc.v);
		cpu.next();
		let line = lines[i as uint].clone();
		let mut it = line.split_str(",");
		let afs = it.next().unwrap();
		let bcs_t = it.next().unwrap();
		let bcs = bcs_t.slice_to(bcs_t.len()-1);
		let afo : Option<u16> = num::from_str_radix(afs, 16);
		let af = afo.unwrap();
		let bco : Option<u16> = num::from_str_radix(bcs, 16);
		let bc = bco.unwrap();
		println!("output js: {:04X}, {:04X}", af, bc);
		println!("output rust: {:04X}, {:04X}", cpu.regs.af.v, cpu.regs.bc.v);
		assert!(cpu.regs.af.v == af);
		assert!(cpu.regs.bc.v == bc);
		cpu.regs.pc.v = 0x100;
	}
}

fn main() {
    sdl::init([sdl::InitVideo]);
    sdl::wm::set_caption("rustgb", "rust-sdl");
    //let screen : ~Surface = match sdl::video::set_video_mode(160, 144, 32, [sdl::video::HWSurface],
    let screen : ~Surface = match sdl::video::set_video_mode(500, 500, 32, [sdl::video::HWSurface],
                                                                [sdl::video::DoubleBuf]) {
        Ok(screen) => screen,
        Err(err) => fail!("failed to set video mode: {}", err)
    };
	let filename = std::os::args()[1];
	let result = match File::open(&Path::new(filename)).read_to_end() {
		Ok(r) => r,
		Err(e) => fail!("failed to read file: {}", e)
	};
	let rom_contents = result.slice_to(result.len()-1);
	let game_title = match std::str::from_utf8(rom_contents.slice(0x134, 0x143)) {
		Some(g) => g,
		None => fail!("Couldn't decode game title")
	};
	//println(game_title);
	let cart_type = rom_contents[0x147];
	//println!("cart type: {:X}", cart_type);
	let mut cpu = Cpu::new(rom_contents.to_owned());
	let mut start_time = current_time_millis();
	let mut events_t = 0;
	let mut draw_t = 0;
	let mut time = 0;
	'main : loop {
		if time < 1000000 {
			time += 1;
		} else {
			let new_time = current_time_millis();
			//println!("t: {}", new_time-start_time);
			start_time = new_time;
			time = 0;
		}
		cpu.next();
		cpu.interrupts();
		cpu.run_clock();
		let lcdc = cpu.mem.readbyte(0xFF40);
		if cpu.drawing && lcdc >> 7 == 1 {
			cpu.drawing = false;
			if draw_t < 10 {
				draw_t += 1;
			} else {
				draw_t = 0;
				draw(screen, cpu.mem.mem.slice(0x8000, 0xA000), lcdc);
				screen.flip();
			}
		}
		if events_t < 1000 {
			events_t += 1;
			continue;
		}
		events_t = 0;
		'events : loop {
			let e = events::events(&mut cpu);
			match e { // Returns false on Quit event
				1 => break 'events,
				2 => break 'main,
				_ => {}
			}
		}
	}
	//println!("t: {}", (current_time_millis()-start_time)/1000);
    sdl::quit();
}


