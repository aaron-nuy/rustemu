use crate::console::constants::*;

pub struct Cartridge {
    rom: [u8; CARTRIDGE_SIZE],
    ram: [u8; MAX_RAM_SIZE],
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: u8,
    rom_size: usize,
    ram_size: usize,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: [0u8; CARTRIDGE_SIZE],
            ram: [0u8; MAX_RAM_SIZE],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: 0,
            rom_size: 0,
            ram_size: MAX_RAM_SIZE, // Default to max for now, or detect later
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.rom_size = data.len();
        let len = data.len().min(CARTRIDGE_SIZE);
        self.rom[..len].copy_from_slice(&data[..len]);

        // Basic RAM size detection from header if possible
        if self.rom_size >= 0x14A {
            let ram_size_idx = self.rom[0x0149];
            self.ram_size = match ram_size_idx {
                0x00 => 0,
                0x01 => 2048,
                0x02 => 8192,
                0x03 => 32768,
                0x04 => 131072,
                0x05 => 65536,
                _ => MAX_RAM_SIZE,
            };
        }
    }

    pub fn read_rom(&self, addr: u16) -> u8 {
        if self.rom_size == 0 { return 0xFF; }
        let rom_len = self.rom_size.min(CARTRIDGE_SIZE);
        match addr {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => {
                let bank = if self.banking_mode == 1 {
                    (self.ram_bank << 5) as usize
                } else {
                    0
                };
                let real_addr = (bank * ROM_BANK_SIZE) | (addr as usize);
                self.rom[real_addr % rom_len]
            }
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => {
                let bank = ((self.ram_bank << 5) | self.rom_bank) as usize;
                let real_addr = (bank * ROM_BANK_SIZE) | ((addr - ROM_BANK_N_BEGIN) as usize);
                self.rom[real_addr % rom_len]
            }
            _ => 0xFF,
        }
    }

    pub fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let mut bank = value & 0x1F;
                if bank == 0 { bank = 1; }
                self.rom_bank = bank;
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x03;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x01;
            }
            _ => {}
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram_size == 0 { return 0xFF; }
        let ram_len = self.ram_size.min(MAX_RAM_SIZE);
        let bank = if self.banking_mode == 1 { self.ram_bank as usize } else { 0 };
        let real_addr = (bank * RAM_BANK_SIZE) | ((addr - EXT_RAM_BEGIN) as usize);
        self.ram[real_addr % ram_len]
    }

    pub fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled || self.ram_size == 0 { return; }
        let ram_len = self.ram_size.min(MAX_RAM_SIZE);
        let bank = if self.banking_mode == 1 { self.ram_bank as usize } else { 0 };
        let real_addr = (bank * RAM_BANK_SIZE) | ((addr - EXT_RAM_BEGIN) as usize);
        self.ram[real_addr % ram_len] = value;
    }
}
