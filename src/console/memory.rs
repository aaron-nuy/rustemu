pub struct Memory {
    ram: Box<[u8; crate::console::constants::MEMORY_SIZE]>,
}

impl Memory {
    pub fn write_to_8b(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    pub fn read_from_8b(&self, address: usize) -> u8 {
        self.ram[address]
    }

    pub fn write_to_16b(&mut self, address: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.ram[address] = bytes[0];
        self.ram[address + 1] = bytes[1];
    }

    pub fn read_from_16b(&self, address: usize) -> u16 {
        let bytes = [self.ram[address], self.ram[address + 1]];
        u16::from_le_bytes(bytes)
    }
}