#![feature(collections)]
#![feature(core)]

extern crate sdl;
extern crate time;
extern crate getopts;

use cpu::Cpu;

use sdl::video::Surface;
use sdl::video::{SurfaceFlag, VideoFlag};
//use sdl::video::Color;
use sdl::video::RGB;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;
use std::cmp::min;
use std::env;
//use getopts::{optflag,getopts,OptGroup,usage};
use getopts::Options;
use time::now_utc;

mod cpu;
mod mem;
mod events;

#[allow(dead_code)]
pub fn current_time_millis() -> u64 {
    let tm = now_utc();
    (tm.tm_sec as u64) * 1000 + (tm.tm_nsec as u64) / 1000
}

fn putpixel(screen : &Surface, x : usize, y : usize, color : u32) {
    screen.lock();
    unsafe {
        let pitch = (*screen.raw).pitch as usize;
        let h = (*screen.raw).h as usize;
        let p = y * (pitch / std::mem::size_of_val(&(0 as u32))) + x;
        let pixels: &mut [u32] = std::mem::transmute(((*screen.raw).pixels, (h*pitch) as usize));
        pixels[p] = color as u32;
    }
    screen.unlock();
}

fn make_tiles(t1 : &Surface, t2 : &Surface, vram : &[u8]) {
    let cols = 16;
    //let base_tiledata_addr = if lcdc >> 4 & 1 == 0 { 0x800 } else { 0 };
    for tile in (0..256) {
	let taddr = tile * 16 + 0x800;
	for line in (0..8) {
	    let laddr = (taddr + 2*line) as usize;
	    for pixel in (0..8) {
		let c = vram[laddr] >> 7 - pixel & 1 |
		(vram[laddr+1] >> 7 - pixel & 1) << 1;
		putpixel(t1, (tile%cols*8+pixel) as usize, (tile/cols*8+line) as usize, match c {
		    3 => 0 as u32,
		    2 => 0x555555 as u32,
		    1 => 0xAAAAAA as u32,
		    0 => 0xFFFFFF as u32,
		    _ => panic!("you are terminated")
		});
	    }
	}
    }
    for tile in (0..256) {
	let taddr = tile * 16;
	for line in (0..8) {
	    let laddr = (taddr + 2*line) as usize;
	    for pixel in (0..8) {
		let c = vram[laddr] >> 7 - pixel & 1 |
		(vram[laddr+1] >> 7 - pixel & 1) << 1;
		putpixel(t2, (tile%cols*8+pixel) as usize, (tile/cols*8+line) as usize, match c {
		    3 => 0 as u32,
		    2 => 0x555555 as u32,
		    1 => 0xAAAAAA as u32,
		    0 => 0xFFFFFF as u32,
		    _ => panic!("you are terminated")
		});
	    }
	}
    }
}

#[allow(unused_variables)]
fn draw_sprites(screen : &Surface, vram : &[u8], oam : &[u8], t1 : &Surface,
                size_8x16 : bool) {
    let cols = 16 as i16;
    for i in (0..40) {
        let base = i * 4;
        let y = oam[base] as i16 - 16;
        let x = oam[base+1] as i16 - 8;
        if 0 > y || y > 144+16 || 0 > x || x > 160+8 {
            continue;
        }
        let mut tile_n = oam[base+2] as i16;
        if size_8x16 {
            tile_n &= 0xFE;
        }
        //println!("i, x, y, tn : {}, {}, {}, {}", i, x, y, tile_n);
        let attrs = oam[base+3];
        let priority = attrs >> 7 & 1;
        // TODO: actually flip
        let yflip = attrs >> 6 & 1;
        let xflip = attrs >> 5 & 1;
        let palnum = attrs >> 4 & 1;

        let sx = tile_n%cols*8;
        let sy = tile_n/cols*8;
        screen.blit_rect(t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
                         Some(sdl::Rect {x:x, y:y, w:8, h:8}));
        if size_8x16 {
            tile_n += 1;
            let sx = tile_n%cols*8;
            let sy = tile_n/cols*8;
            screen.blit_rect(t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
                             Some(sdl::Rect {x:x, y:y + 8, w:8, h:8}));
        }
    }
}

fn draw_frame(screen : &Surface) {
    screen.fill_rect(Some(sdl::Rect {x: 0, y: 145, w: 160, h: 1}),
		     RGB(0xFF, 0x0, 0xFF));
    screen.fill_rect(Some(sdl::Rect {x: 0, y: 257, w: 256, h: 1}),
		     RGB(0xFF, 0x0, 0xFF));
}

fn draw(screen : &Surface, vram : &[u8], oam : &[u8], lcdc : u8) {
    screen.fill_rect(Some(sdl::Rect {x: 0, y: 0, w: 160, h: 144}),
		     RGB(0xFF, 0xFF, 0xFF));
    let t1 = match Surface::new(&[], 512, 512, 32, 0x00ff0000, 0x0000ff00, 0x000000ff, 0) {
        Ok(s) => s,
        Err(err) => panic!("failed to create surface: {}", err)
    };
    let t2 = match Surface::new(&[], 512, 512, 32, 0x00ff0000, 0x0000ff00, 0x000000ff, 0) {
        Ok(s) => s,
        Err(err) => panic!("failed to create surface: {}", err)
    };
    make_tiles(&t1, &t2, vram);
    let cols = 16;
    let base_bgmap_addr = if lcdc >> 3 & 1 == 0 { 0x1800 } else { 0x1C00 };
    let base_window_addr = if lcdc >> 6 & 1 == 0 { 0x1800 } else { 0x1C00 };

    let t = if lcdc >> 4 & 1 == 0 { &t1 } else { &t2 };
    //let t = &t1;

    // BG
    for row in (0..32) {
	for cell_n in (0..32) {
	    let addr = (base_bgmap_addr+row*32+cell_n) as usize;
            let mut tile_n : i16;
	    if lcdc >> 4 & 1 == 0 {
                //tile_n = (vram as &[i8])[addr] as i16;
                tile_n = vram[addr] as i8 as i16;
		tile_n += 128;
	    } else {
                tile_n = vram[addr] as i16;
            }
	    let sx = tile_n%(cols as i16)*8;
	    let sy = tile_n/(cols as i16)*8;
	    screen.blit_rect(t, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
			     Some(sdl::Rect {x:(cell_n*8) as i16, y:(row*8) as i16, w:8, h:8}));
	}
    }

    // Window
    // TODO - dunno what to do. kirby seems to enable it but it shouldn't
    let window_enabled = lcdc >> 5 & 1 == 1 && false;
    if window_enabled {
        for row in (0..32) {
            for cell_n in (0..32) {
                let addr = (base_window_addr+row*32+cell_n) as usize;
                let mut tile_n : i16;
                if lcdc >> 4 & 1 == 0 {
                    //tile_n = (vram as &[i8])[addr] as i16;
                    tile_n = vram[addr] as i8 as i16;
                    tile_n += 128;
                } else {
                    tile_n = vram[addr] as i16;
                }
                let sx = tile_n%(cols as i16)*8;
                let sy = tile_n/(cols as i16)*8;
                screen.blit_rect(t, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
                                 Some(sdl::Rect {x:(cell_n*8) as i16, y:(row*8) as i16, w:8, h:8}));
            }
        }
    }

    if (lcdc >> 1 & 1) == 1 {
        let sprite_size = (lcdc >> 2 & 1) == 1;
        draw_sprites(screen, vram, oam, &t2, sprite_size);
    }
    screen.blit_at(&t1, 0, 258);
    screen.blit_at(&t2, 256, 258);
    draw_frame(screen);
}

#[test]
fn test_instr() {
    //let mut f = BufferedReader::new(File::open(&Path::new("out")));
    //let lines: ~[~str] = f.lines().map(|x| x.unwrap()).collect();

    let mut rom = [0 as u8, ..0x200];
    rom[0x100] = 0x27;
    let mut cpu = Cpu::new(rom);
    for i_ in (0..0xFFFF) {
	let i = i_ as u16;
	cpu.regs.af.v = i & 0xFFF0;
	cpu.regs.bc.v = 0;
	//cpu.regs.af.v = (i << 8 | 1 << 4);
	//cpu.regs.bc.v = (i & 0xFF00);
	println!("input: {:04X}, {:04X}", cpu.regs.af.v, cpu.regs.bc.v);
	cpu.next();
	//let line = lines[i as usize].clone();
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

fn print_usage(program: &str, opts: Options) {
    let descr = "Emulates a gameboy ROM image in FILE.";
    let help = format!("Usage: {} [OPTION]... [FILE]\n{}", program, descr);
    print!("{}", opts.usage(&help));
}

fn main() {
    let args : Vec<String> = env::args().collect();
    let program_name = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu");
    let matches = match opts.parse(args.tail()) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program_name[..], opts);
        return;
    }
    if matches.free.is_empty() {
        println!("Error: What ROM do you want to emulate?");
        print_usage(&program_name[..], opts);
        return;
    }

    sdl::init(&[sdl::InitFlag::Video][..]);
    sdl::wm::set_caption("rustgb", "rust-sdl");
    //let screen : ~Surface = match sdl::video::set_video_mode(160, 144, 32, [sdl::video::HWSurface],
    let screen : Box<Surface> = match sdl::video::set_video_mode(500, 500, 32,
                                                                 &[SurfaceFlag::HWSurface][..],
                                                                 &[VideoFlag::DoubleBuf][..]) {
        Ok(screen) => Box::new(screen),
        Err(err) => panic!("failed to set video mode: {}", err)
    };

    let filename = &args[1][..];
    let path = Path::new(filename);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(),
                           Error::description(&why)),
        Ok(file) => file,
    };
    let mut buf : Vec<u8> = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => 0,
        Err(e) => panic!("failed to read file: {}", e)
    };
    //let rom_contents = result.slice_to(result.len()-1);
    let rom_contents : &[u8] = &buf[..min(0x100000, buf.len()-1)];
    let game_title = match std::str::from_utf8(&rom_contents[0x134..0x143]) {
        Ok(g) => g,
        Err(_) => "UNKNOWN"
    };
    println!("{}", game_title);
    let cart_type = rom_contents[0x147];
    println!("cart type: {:X}", cart_type);
    let mut cpu = Cpu::new(rom_contents, args.len() > 2);
    //let mut start_time = current_time_millis();
    let mut events_t = 0;
    let mut draw_t = 0;
    //let mut time = 0;
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
                draw(&*screen, &cpu.mem.mem[0x8000..0xA000], &cpu.mem.mem[0xFE00..0xFEA0], lcdc);
                screen.flip();
            }
        }
        if events_t < 10 {
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


