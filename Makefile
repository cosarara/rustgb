main: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl -A dead-code main.rs

main_quiet: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl -A unused-mut -A unused-variable -A dead-code main.rs

test: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl -A unused-mut -A unused-variable -A dead-code --test main.rs

clean:
	rm main

all: main
