struct Cartridge {
    ram: Box<[u8; crate::constants::CARTRIDGE_SIZE]>,
}