
extern crate sdl;
use cpu::Cpu;

pub fn events(cpu : &mut Cpu) -> int {
	match sdl::event::poll_event() {
		sdl::event::NoEvent => return 1,
		sdl::event::QuitEvent => return 2,
		sdl::event::KeyEvent(k, p, _, _) => {
			if k == sdl::event::EscapeKey {
				return 2
			} else if k == sdl::event::ReturnKey {
				cpu.mem.kstart = p
			} else if k == sdl::event::BackspaceKey {
				cpu.mem.kselect = p
			} else if k == sdl::event::ZKey {
				cpu.mem.ka = p
			} else if k == sdl::event::XKey {
				cpu.mem.kb = p
			} else if k == sdl::event::UpKey {
				cpu.mem.kup = p
			} else if k == sdl::event::DownKey {
				cpu.mem.kdown = p
			} else if k == sdl::event::RightKey {
				cpu.mem.kright = p
			} else if k == sdl::event::LeftKey {
				cpu.mem.kleft = p
			}
		},
		_ => {}
	}
	0
}
