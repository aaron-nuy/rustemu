pub struct Memory {
    ram: Box<[u8; crate::console::constants::MEMORY_SIZE]>,
}

impl Memory {
    pub fn write_to_8b(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }

    pub fn read_from_8b(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_to_16b(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.ram[address as usize] = bytes[0];
        self.ram[address.wrapping_add(1) as usize] = bytes[1];
    }

    pub fn read_from_16b(&self, address: u16) -> u16 {
        let bytes = [self.ram[address as usize], self.ram[address.wrapping_add(1) as usize]];
        u16::from_le_bytes(bytes)
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.ram[..data.len()].copy_from_slice(&data);
    }

    pub fn new() -> Self {
        Self {
            ram: Box::new([0; crate::console::constants::MEMORY_SIZE])
        }
    }
}