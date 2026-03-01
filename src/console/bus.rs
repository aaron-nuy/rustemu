use crate::console::audio::Audio;
use crate::console::cartridge::Cartridge;
use crate::console::constants::*;
use crate::console::gui::gpu::Gpu;
use crate::console::hw_register::{HwRegisterAddr, HwRegisters};
use crate::console::interrupt::Interrupt;

pub struct Bus {
    ram: Box<[u8; MEMORY_SIZE]>,
    boot_rom: [u8; BOOT_ROM_SIZE],
    boot_rom_enabled: bool,
    cartridge: Cartridge,
    pub gpu: Gpu,
    audio: Audio,
    hw_registers: HwRegisters,
}

impl Bus {
    fn write_to_bus(&mut self, addr: u16, value: u8) {
        match addr {
            addr if addr == BOOT_ROM_DISABLE_ADDR => {
                self.boot_rom_enabled = false;
            }
            addr if HwRegisters::supported_addr(addr) => self.hw_registers.set_addr(addr, value),
            VRAM_BEGIN..=VRAM_END => {
                self.gpu.write_to_vram(addr - VRAM_BEGIN, value);
            }
            _ => self.ram[addr as usize] = value,
        }
    }

    fn read_from_bus(&self, addr: u16) -> u8 {
        match addr {
            addr if self.boot_rom_enabled && addr < BOOT_ROM_SIZE as u16 => {
                self.boot_rom[addr as usize]
            }
            addr if HwRegisters::supported_addr(addr) => self.hw_registers.get_addr(addr),
            VRAM_BEGIN..=VRAM_END => self.gpu.read_from_vram(addr - VRAM_BEGIN),
            _ => self.ram[addr as usize],
        }
    }

    pub fn write_to_8b(&mut self, addr: u16, value: u8) {
        self.write_to_bus(addr, value);
    }

    pub fn read_from_8b(&self, addr: u16) -> u8 {
        self.read_from_bus(addr)
    }

    pub fn write_to_16b(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.write_to_bus(addr, bytes[0]);
        self.write_to_bus(addr.wrapping_add(1), bytes[1]);
    }

    pub fn read_from_16b(&self, addr: u16) -> u16 {
        let bytes = [
            self.read_from_bus(addr),
            self.read_from_bus(addr.wrapping_add(1)),
        ];
        u16::from_le_bytes(bytes)
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        let rom_max_len = 0x8000usize;
        let rom_len = data.len().min(rom_max_len);

        self.ram[..rom_len].copy_from_slice(&data[..rom_len]);

        self.gpu.vram.fill(0);
    }

    pub fn get_interrupt(&self) -> Option<(Interrupt, u16)> {
        let _ie = self.hw_registers.get_hw_register(HwRegisterAddr::IE);
        let _if = self.hw_registers.get_hw_register(HwRegisterAddr::IF);
        let interrupt_mask = _ie & _if;

        Interrupt::get_interrupt(interrupt_mask)
    }

    pub fn unset_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.hw_registers.get_hw_register(HwRegisterAddr::IF);

        _if &= !(interrupt as u8);

        self.hw_registers.set_hw_register(HwRegisterAddr::IF, _if);
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.hw_registers.get_hw_register(HwRegisterAddr::IF);

        _if |= interrupt as u8;

        self.hw_registers.set_hw_register(HwRegisterAddr::IF, _if);
    }

    pub fn inc_div(&mut self) {
        self.hw_registers.inc_div()
    }

    pub fn new() -> Self {
        Self {
            ram: Box::new([0; MEMORY_SIZE]),
            gpu: Gpu::new(),
            audio: Audio::new(),
            cartridge: Cartridge::new(),
            hw_registers: HwRegisters::default(),
            boot_rom: BOOT_ROM,
            boot_rom_enabled: true,
        }
    }
}
