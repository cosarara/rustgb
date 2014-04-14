
extern crate sdl;
use sdl::video::Surface;
use sdl::video::Color;
use sdl::video::RGB;
use std::io::File;
use cpu::Cpu;
use std::io::println;
mod cpu;
mod mem;

fn draw(screen : &Surface, vram : &[u8]) {
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
	for tile in range(0, 255) {
		let taddr = tile * 16;
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
	for row in range(0, 32) {
		for cell_n in range(0, 32) {
			let addr = (0x1800+row*32+cell_n) as uint;
			let tile_n = vram[addr] as i16;
			let sx = tile_n%(cols as i16)*8;
			let sy = tile_n/(cols as i16)*8;
			screen.blit_rect(t1, Some(sdl::Rect {x:sx, y:sy, w:8, h:8}),
				Some(sdl::Rect {x:(cell_n*8) as i16, y:(row*8) as i16, w:8, h:8}));
		}
	}
	screen.blit_at(t1, 0, 260);
	putpixel(screen, 5, 5, RGB(0xFF, 0, 0));
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
	println(game_title);
	let cart_type = rom_contents[0x147];
	println!("cart type: {}", cart_type);
	let mut cpu = Cpu::new(rom_contents.to_owned());
	'main : loop {
		cpu.next();
		if cpu.drawing {
			draw(screen, cpu.mem.mem.slice(0x8000, 0xA000));
			screen.flip();
			cpu.drawing = false;
		}
        'events : loop {
			match sdl::event::poll_event() {
				sdl::event::NoEvent => break 'events,
				sdl::event::QuitEvent => break 'main,
				//sdl::event::KeyEvent(k, _, _, _)
				//	if k == sdl::event::EscapeKey
				//		=> break 'main,
				_ => {}
			}
		}
	}
    sdl::quit();
}


