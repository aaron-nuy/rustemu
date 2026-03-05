#[cfg(not(efi))]
pub mod args {
    use lexopt::Arg::{Long, Short};
    use lexopt::{Parser, ValueExt};
    use std::process;

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

    pub fn parse_args() -> Result<(Option<[u32; 4]>, Option<String>), String> {
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
}

#[cfg(efi)]
pub mod args {
    // Not implemented
}
