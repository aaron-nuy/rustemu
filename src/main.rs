use std::env;
use console::gameboy::{Gameboy};

mod console;


fn main() {
    let args: Vec<String> = env::args().collect();

    let mut gameboy = Gameboy::new();

    gameboy.load(&args[1]);

    gameboy.run();
}
