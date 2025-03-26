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
}