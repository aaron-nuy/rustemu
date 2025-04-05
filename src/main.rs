use console::gameboy::{Gameboy};

mod console;


fn main() {
    let mut gameboy = Gameboy::new();

    gameboy.load("10-bit ops.gb");

    gameboy.run();
}
