pub struct Bus {
    ram: Box<[u8; crate::console::constants::MEMORY_SIZE]>,
}

impl Bus {
    const SB_ADR: u16 = 0xFF01;
    const SC_ADR: u16 = 0xFF02;

    fn write_to_bus(&mut self, address: u16, value: u8) {

        match address {
            0xFF02 => {
                self.ram[Bus::SC_ADR] = value;

                if (value & 0x80) != 0 {
                    print!("{}", self.ram[Bus::SB_ADR] as char);
                    self.ram[Bus::SC_ADR] &= 0x7F;
                }
            }
            _ => {
                self.ram[address as usize] = value;
            }
        }
    }

    fn read_from_bus(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_to_8b(&mut self, address: u16, value: u8) {
        self.write_to_bus(address, value);
    }

    pub fn read_from_8b(&self, address: u16) -> u8 {
        self.read_from_bus(address)
    }

    pub fn write_to_16b(&mut self, address: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_to_bus(address, bytes[0]);
        self.write_to_bus(address.wrapping_add(1), bytes[1]);
    }

    pub fn read_from_16b(&self, address: u16) -> u16 {
        let bytes = [self.read_from_bus(address), self.read_from_bus(address.wrapping_add(1))];
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