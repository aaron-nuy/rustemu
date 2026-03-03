use console::gameboy::Gameboy;
use std::env;

mod console;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut rom_file = "dr_mario.gb".to_string();
    //let mut rom_file = "test_roms/01-special.gb".to_string();
    //let mut rom_file = "test_roms/02-interrupts.gb".to_string();
    //let mut rom_file = "test_roms/03-op sp,hl.gb".to_string();
    //let mut rom_file = "test_roms/03-op sp,hl.gb".to_string();
    //let mut rom_file = "test_roms/04-op r,imm.gb".to_string();
    //let mut rom_file = "test_roms/05-op rp.gb".to_string();
    //let mut rom_file = "test_roms/06-ld r,r.gb".to_string();
    //let mut rom_file = "test_roms/07-jr,jp,call,ret,rst.gb".to_string();
    //let mut rom_file = "test_roms/08-misc instrs.gb".to_string();
    //let mut rom_file = "test_roms/09-op r,r.gb".to_string();
    //let mut rom_file = "test_roms/10-bit ops.gb".to_string();
    //let mut rom_file = "test_roms/11-op a,(hl).gb".to_string();
    if args.len() >= 2 {
        rom_file = args[1].clone();
    }

    let mut gameboy = Gameboy::new();

    gameboy.load(&rom_file);

    gameboy.run();
}
