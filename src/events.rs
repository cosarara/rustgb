
extern crate sdl;
use cpu::Cpu;
use sdl::event::{Event, Key};

pub fn events(cpu : &mut Cpu) -> isize {
    match sdl::event::poll_event() {
        Event::None => return 1,
        Event::Quit => return 2,
        Event::Key(k, p, _, _) => {
            if k == Key::Escape {
                return 2
            } else if k == Key::Return {
                cpu.mem.kstart = p
            } else if k == Key::Backspace {
                cpu.mem.kselect = p
            } else if k == Key::Z {
                cpu.mem.ka = p
            } else if k == Key::X {
                cpu.mem.kb = p
            } else if k == Key::Up {
                cpu.mem.kup = p
            } else if k == Key::Down {
                cpu.mem.kdown = p
            } else if k == Key::Right {
                cpu.mem.kright = p
            } else if k == Key::Left {
                cpu.mem.kleft = p
            }
        },
        _ => {}
    }
    0
}
