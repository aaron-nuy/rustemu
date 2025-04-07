use std::env;
use std::process::exit;
use console::gameboy::{Gameboy};

mod console;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} [rom file]", args[0]);
        exit(2);
    }

    let rom_file = &args[1];

    let mut gameboy = Gameboy::new();

    gameboy.load(rom_file);

    gameboy.run();
}
