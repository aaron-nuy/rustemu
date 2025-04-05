pub struct Cartridge {
    rom: Box<[u8; crate::console::constants::CARTRIDGE_SIZE]>,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: Box::new([0;crate::console::constants::CARTRIDGE_SIZE])
        }
    }
}