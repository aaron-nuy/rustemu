use console::gameboy::Gameboy;
use lexopt::Parser;
use lexopt::prelude::*;
use std::process;
use std::process::exit;

mod console;

fn main() {
    //let mut rom_file = "dr_mario.gb".to_string();
    //let mut rom_file = "tetris.gb".to_string();
    //let mut rom_file = "alleyway.gb".to_string();
    //let mut rom_file = "dmg-acid2.gb".to_string();
    //let rom_file = "kroyo.gb".to_string();
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

    match parse_args() {
        Ok((palette_opt, rom_file_opt)) => {
            let palette_args: Option<[u32; 4]> = palette_opt;
            let rom_file_args: Option<String> = rom_file_opt;

            let rom = match rom_file_args {
                Some(rom_file_o) => rom_file_o.to_string(),
                None => {
                    eprintln!("No romfile selected");
                    exit(1);
                }
            };

            let mut gameboy = match palette_args {
                Some([z, o, t, tr]) => Gameboy::new_with_pal(z, o, t, tr),
                None => Gameboy::new(),
            };

            gameboy.load(&rom);

            gameboy.run();
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(2);
        }
    }
}

fn parse_u32_lenient(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16).map_err(|e| format!("invalid hex '{}': {}", s, e))
    } else {
        s.parse::<u32>()
            .or_else(|_| u32::from_str_radix(s, 16))
            .map_err(|e| format!("invalid number '{}': {}", s, e))
    }
}

fn print_usage_and_exit(program: &str) -> ! {
    eprintln!(
        "Usage: {prog} [--palette <a> <b> <c> <d>] [--rom_file]
  --palette   four u32 values (decimal, 0xhex, or plain hex digits)
  --rom_file    optional positional ROM file path
  -h, --help  show this message",
        prog = program
    );
    process::exit(2);
}

fn parse_palette(parser: &mut Parser) -> Result<[u32; 4], String> {
    let mut vals = [0u32; 4];
    for i in 0..4 {
        let raw: String = parser
            .value()
            .map_err(|e| format!("expected palette value {}: {}", i + 1, e))?
            .parse()
            .map_err(|e| e.to_string())?;
        vals[i] = parse_u32_lenient(&raw)?;
    }
    Ok(vals)
}

fn parse_args() -> Result<(Option<[u32; 4]>, Option<String>), String> {
    let mut parser = Parser::from_env();
    let program = std::env::args().next().unwrap_or_else(|| "program".into());

    let mut palette: Option<[u32; 4]> = None;
    let mut rom_file: Option<String> = None;

    while let Some(arg) = parser.next().map_err(|e| e.to_string())? {
        match arg {
            Long("palette") => {
                if palette.is_some() {
                    return Err("--palette specified multiple times".into());
                }
                palette = Some(parse_palette(&mut parser)?);
            }
            Short('h') | Long("help") => print_usage_and_exit(&program),
            Long("rom_file") => {
                if rom_file.is_some() {
                    return Err("--rom_file specified multiple times".into());
                }

                rom_file = Some(
                    parser
                        .value()
                        .map_err(|e| e.to_string())?
                        .parse()
                        .map_err(|e| e.to_string())?,
                );
            }
            _ => return Err(arg.unexpected().to_string().into()),
        }
    }

    Ok((palette, rom_file))
}
