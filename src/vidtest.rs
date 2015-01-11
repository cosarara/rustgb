
extern crate sdl;
use sdl::video::Surface;
mod events_test;

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

fn main() {
	let screen : Box<Surface> = match sdl::video::set_video_mode(500, 500, 32, [sdl::video::HWSurface],
	                                                            [sdl::video::DoubleBuf]) {
	    Ok(screen) => box screen,
	    Err(err) => fail!("failed to set video mode: {}", err)
	};
    putpixel(screen, 5, 5, 0xAAAAAA as u32);
    'main : loop {
        screen.flip();
        'events : loop {
            let e = events_test::events();
            match e { // Returns false on Quit event
                1 => break 'events,
                2 => break 'main,
                _ => {}
            }
        }
    }
}
