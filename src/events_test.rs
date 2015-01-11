
extern crate sdl;

pub fn events() -> int {
	match sdl::event::poll_event() {
		sdl::event::NoEvent => return 1,
		sdl::event::QuitEvent => return 2,
		_ => {}
	}
	0
}
