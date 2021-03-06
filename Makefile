main: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl main.rs

fast: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl -O main.rs

main_quiet: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl -A unused-mut -A unused-variable -A dead-code main.rs

tests: main.rs mem.rs cpu.rs
	rustc -Lrust-sdl --test main.rs

clean:
	rm main

all: main

%.o : %.asm
	rgbasm $< -o $@

%.gb : %.o
	rgblink $< -o $@
	rgbfix $@

