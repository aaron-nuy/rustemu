#![cfg_attr(efi, no_main)]
#![cfg_attr(efi, no_std)]

use log::info;
use console::gameboy::Gameboy;

mod arg_parse;
mod console;
mod read_rom;

#[cfg(efi)]
use uefi::{entry, Status};

#[cfg(efi)]
#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("Initializing Gameboy");
    let mut gameboy = Gameboy::new();

    info!("Opening file: {}", "default.gb");
    gameboy.load(&"default.gb");
    gameboy.run();

    Status::SUCCESS
}

#[cfg(not(efi))]
fn main() {
    use std::process::exit;

    let (rom_file, palette) = match arg_parse::args::parse_args() {
        Ok((palette_opt, Some(f))) => (f, palette_opt),
        Ok((_, None)) => {
            eprintln!("No romfile selected");
            exit(1);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    };

    let mut gameboy = match palette {
        Some([z, o, t, tr]) => Gameboy::new_with_pal(z, o, t, tr),
        None => Gameboy::new(),
    };


    gameboy.load(&rom_file);
    gameboy.run();
}