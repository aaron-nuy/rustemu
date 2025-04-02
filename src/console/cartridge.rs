pub struct Cartridge {
    ram: Box<[u8; crate::console::constants::CARTRIDGE_SIZE]>,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            ram: Box::new([0;crate::console::constants::CARTRIDGE_SIZE])
        }
    }
}