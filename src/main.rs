use console::gameboy::Gameboy;
use std::env;

mod console;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut rom_file = "dr_mario.gb".to_string();
    //let mut rom_file = "test_roms/01-special.gb".to_string();
    if args.len() >= 2 {
        rom_file = args[1].clone();
    }

    let mut gameboy = Gameboy::new();

    gameboy.load(&rom_file);

    gameboy.run();
}
