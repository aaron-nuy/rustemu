pub struct Cartridge {
    ram: Box<[u8; crate::console::constants::CARTRIDGE_SIZE]>,
}