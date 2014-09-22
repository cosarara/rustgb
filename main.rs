
extern crate sdl;
extern crate time;

use cpu::Cpu;

use sdl::video::Surface;
//use sdl::video::Color;
use sdl::video::RGB;
use std::io::File;
use std::io::println;
use std::cmp::min;
use time::now_utc;

mod cpu;
mod mem;
mod events;

#[allow(dead_code)]
pub fn current_time_millis() -> u64 {
	let tm = now_utc();
	(tm.tm_sec as u64) * 1000 + (tm.tm_nsec as u64) / 1000
}

fn putpixel(screen : &Surface, x : uint, y : uint, color : u32) {
    screen.lock();
    unsafe {
        let pitch = (*screen.raw).pitch as uint;
        let h = (*screen.raw).h as uint;
        let p = y * (pitch / std::mem::size_of_val(&(0 as u32))) + x;
        let pixels: &mut [u32] = std::mem::transmute(((*screen.raw).pixels, (h*pitch) as uint));
        pixels[p] = color as u32;
    }
    screen.unlock();
}

#[allow(unused_variable)]
fn draw_sprites(screen : &Surface, vram : &[u8], oam : &[u8], t1 : &Surface) {
	let cols = 16u;
    for i in range(0u, 40) {
        let base = i * 4;
        let y = oam[base] - 16;
        let x = oam[base+1] - 8;
        //println!("x, y : {}, {}, {}", x, y, oam[0]);
        let tile_n = oam[base+2] as i16;
        let attrs = oam[base+3];
        let priority = attrs >> 7 & 1;
        let yflip = attrs >> 6 & 1;
        let xflip = attrs >> 5 & 1;
        let palnum = attrs >> 4 & 1;

        let sx = tile_n%(cols as i16)*8;
        let sy = tile_n/(cols as i16)*8;
        screen.blit_rect(t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
            Some(sdl::Rect {x:x as i16, y:y as i16, w:8, h:8}));
    }
}

fn draw(screen : &Surface, vram : &[u8], oam : &[u8], lcdc : u8) {
	screen.fill_rect(Some(sdl::Rect {x: 0, y: 0, w: 160, h: 140}),
		RGB(0xFF, 0xFF, 0xFF));
	let t1 = match Surface::new(&[], 512, 512, 32, 0x00ff0000, 0x0000ff00, 0x000000ff, 0) {
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
				putpixel(&t1, (tile%cols*8+pixel) as uint, (tile/cols*8+line) as uint, match c {
					3 => 0 as u32,
					2 => 0x555555 as u32,
					1 => 0xAAAAAA as u32,
					0 => 0xFFFFFF as u32,
					_ => fail!("you are terminated")
				});
			}
		}
	}
	let base_bgmap_addr = if lcdc >> 3 & 1 == 0 { 0x1800 } else { 0x1C00 };
	for row in range(0u, 32) {
		for cell_n in range(0u, 32) {
			let addr = (base_bgmap_addr+row*32+cell_n) as uint;
			let mut tile_n = vram[addr] as i16;
			if lcdc >> 4 & 1 == 0 {
				tile_n = (tile_n as u8) as i16;
				tile_n += 128;
			}
			let sx = tile_n%(cols as i16)*8;
			let sy = tile_n/(cols as i16)*8;
			screen.blit_rect(&t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
				Some(sdl::Rect {x:(cell_n*8) as i16, y:(row*8) as i16, w:8, h:8}));
		}
	}
    draw_sprites(screen, vram, oam, &t1);
	screen.blit_at(&t1, 0, 260);
}

#[test]
fn test_instr() {
	//let mut f = BufferedReader::new(File::open(&Path::new("out")));
	//let lines: ~[~str] = f.lines().map(|x| x.unwrap()).collect();

	let mut rom = [0 as u8, ..0x200];
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
		//let line = lines[i as uint].clone();
		//let mut it = line.split_str(",");
		//let afs = it.next().unwrap();
		//let bcs_t = it.next().unwrap();
		//let bcs = bcs_t.slice_to(bcs_t.len()-1);
		//let afo : Option<u16> = num::from_str_radix(afs, 16);
		//let af = afo.unwrap();
		//let bco : Option<u16> = num::from_str_radix(bcs, 16);
		//let bc = bco.unwrap();
		//println!("output js: {:04X}, {:04X}", af, bc);
		//println!("output rust: {:04X}, {:04X}", cpu.regs.af.v, cpu.regs.bc.v);
		//assert!(cpu.regs.af.v == af);
		//assert!(cpu.regs.bc.v == bc);
		//cpu.regs.pc.v = 0x100;
	}
}

fn main() {
	sdl::init([sdl::InitVideo]);
	sdl::wm::set_caption("rustgb", "rust-sdl");
	//let screen : ~Surface = match sdl::video::set_video_mode(160, 144, 32, [sdl::video::HWSurface],
	let screen : Box<Surface> = match sdl::video::set_video_mode(500, 500, 32, [sdl::video::HWSurface],
	                                                            [sdl::video::DoubleBuf]) {
	    Ok(screen) => box screen,
	    Err(err) => fail!("failed to set video mode: {}", err)
	};
    	
	let args = std::os::args();
	let filename = args[1].as_slice();
	let path = Path::new(filename);
	let mut file = match File::open(&path) {
		Err(why) => fail!("couldn't open {}: {}", path.display(), why.desc),
		Ok(file) => file,
	};
	let result = match file.read_to_end() {
		Ok(r) => r,
		Err(e) => fail!("failed to read file: {}", e)
	};
	//let rom_contents = result.slice_to(result.len()-1);
	let rom_contents : &[u8] = result.slice_to(min(0x100000, result.len()-1));
	let game_title = match std::str::from_utf8(rom_contents.slice(0x134, 0x143)) {
		Some(g) => g,
		None => fail!("Couldn't decode game title")
	};
	println(game_title);
	let cart_type = rom_contents[0x147];
	println!("cart type: {:X}", cart_type);
	let mut cpu = Cpu::new(rom_contents);
	//let mut start_time = current_time_millis();
	let mut events_t = 0u;
	let mut draw_t = 0u;
	//let mut time = 0u;
	'main : loop {
		/*
		if time < 1000000 {
			time += 1;
		} else {
			let new_time = current_time_millis();
			// This should print something close to 1000
			//println!("t: {}", new_time-start_time);
			//start_time = new_time;
			time = 0;
		}
		*/
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
                // dereference and get pointer again ? WTF rust
				draw(&*screen, cpu.mem.mem.slice(0x8000, 0xA000), cpu.mem.mem.slice(0xFE00, 0xFEA0), lcdc);
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


