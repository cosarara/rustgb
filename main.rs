
extern crate sdl;
use std::io::File;
use cpu::Cpu;
use std::io::println;
mod cpu;
mod mem;

fn main() {
    sdl::init([sdl::InitVideo]);
    sdl::wm::set_caption("rust-sdl demo - video", "rust-sdl");
    let screen = match sdl::video::set_video_mode(160, 144, 32, [sdl::video::HWSurface],
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
	cpu.run();

    screen.flip();
    sdl::quit();
}


