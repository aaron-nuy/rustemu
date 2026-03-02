use crate::console::audio::Audio;
use crate::console::cartridge::Cartridge;
use crate::console::constants::*;
use crate::console::gui::gpu::Gpu;
use crate::console::hw_register::HwRegister;
use crate::console::hw_register::HwRegisters;
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
            addr if HwRegister::supported_addr(addr) => {
                use HwRegister::*;
                let hw_register_addr = HwRegister::from_addr(addr);
                match hw_register_addr {
                    P1 => self.hw_registers._p1 = value,
                    SB => self.hw_registers._sb = value,
                    SC => {
                        self.hw_registers._sc = value;

                        if (value & 0x80) != 0 {
                            print!("{}", self.hw_registers._sb as char);
                            self.hw_registers._sc &= 0x7F;
                        }
                    }
                    DIV => self.hw_registers._div = 0x00,
                    TIMA => self.hw_registers._tima = value,
                    TMA => self.hw_registers._tma = value,
                    TAC => {
                        self.inc_tima();
                        self.hw_registers._tac = value
                    }
                    IF => self.hw_registers._if = value,
                    NR10 => self.hw_registers._nr10 = value,
                    NR11 => self.hw_registers._nr11 = value,
                    NR12 => self.hw_registers._nr12 = value,
                    NR13 => self.hw_registers._nr13 = value,
                    NR14 => self.hw_registers._nr14 = value,
                    NR21 => self.hw_registers._nr21 = value,
                    NR22 => self.hw_registers._nr22 = value,
                    NR23 => self.hw_registers._nr23 = value,
                    NR24 => self.hw_registers._nr24 = value,
                    NR30 => self.hw_registers._nr30 = value,
                    NR31 => self.hw_registers._nr31 = value,
                    NR32 => self.hw_registers._nr32 = value,
                    NR33 => self.hw_registers._nr33 = value,
                    NR34 => self.hw_registers._nr34 = value,
                    NR41 => self.hw_registers._nr41 = value,
                    NR42 => self.hw_registers._nr42 = value,
                    NR43 => self.hw_registers._nr43 = value,
                    NR44 => self.hw_registers._nr44 = value,
                    NR50 => self.hw_registers._nr50 = value,
                    NR51 => self.hw_registers._nr51 = value,
                    NR52 => self.hw_registers._nr52 = value,
                    LCDC => self.hw_registers._lcdc = value,
                    STAT => self.hw_registers._stat = value,
                    SCY => self.hw_registers._scy = value,
                    SCX => self.hw_registers._scx = value,
                    LY => self.hw_registers._ly = value,
                    LYC => self.hw_registers._lyc = value,
                    DMA => {
                        self.hw_registers._dma = value;
                        
                    }
                    BGP => self.hw_registers._bgp = value,
                    OBP0 => self.hw_registers._obp0 = value,
                    OBP1 => self.hw_registers._obp1 = value,
                    WY => self.hw_registers._wy = value,
                    WX => self.hw_registers._wx = value,
                    IE => self.hw_registers._ie = value,
                }
            }
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
            addr if HwRegister::supported_addr(addr) => {
                use HwRegister::*;
                let hw_register_addr = HwRegister::from_addr(addr);
                match hw_register_addr {
                    P1 => self.hw_registers._p1,
                    SB => self.hw_registers._sb,
                    SC => self.hw_registers._sc,
                    DIV => self.hw_registers._div,
                    TIMA => self.hw_registers._tima,
                    TMA => self.hw_registers._tma,
                    TAC => self.hw_registers._tac,
                    IF => self.hw_registers._if,
                    NR10 => self.hw_registers._nr10,
                    NR11 => self.hw_registers._nr11,
                    NR12 => self.hw_registers._nr12,
                    NR13 => self.hw_registers._nr13,
                    NR14 => self.hw_registers._nr14,
                    NR21 => self.hw_registers._nr21,
                    NR22 => self.hw_registers._nr22,
                    NR23 => self.hw_registers._nr23,
                    NR24 => self.hw_registers._nr24,
                    NR30 => self.hw_registers._nr30,
                    NR31 => self.hw_registers._nr31,
                    NR32 => self.hw_registers._nr32,
                    NR33 => self.hw_registers._nr33,
                    NR34 => self.hw_registers._nr34,
                    NR41 => self.hw_registers._nr41,
                    NR42 => self.hw_registers._nr42,
                    NR43 => self.hw_registers._nr43,
                    NR44 => self.hw_registers._nr44,
                    NR50 => self.hw_registers._nr50,
                    NR51 => self.hw_registers._nr51,
                    NR52 => self.hw_registers._nr52,
                    LCDC => self.hw_registers._lcdc,
                    STAT => self.hw_registers._stat,
                    SCY => self.hw_registers._scy,
                    SCX => self.hw_registers._scx,
                    LY => self.hw_registers._ly,
                    LYC => self.hw_registers._lyc,
                    DMA => self.hw_registers._dma,
                    BGP => self.hw_registers._bgp,
                    OBP0 => self.hw_registers._obp0,
                    OBP1 => self.hw_registers._obp1,
                    WY => self.hw_registers._wy,
                    WX => self.hw_registers._wx,
                    IE => self.hw_registers._ie,
                }
            }
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
        let rom_len = data.len().min(CARTRIDGE_SIZE);

        self.ram[..rom_len].copy_from_slice(&data[..rom_len]);

        self.gpu.vram.fill(0);
    }

    pub fn fill_cartridge(&mut self, data: Vec<u8>) {
        self.cartridge.rom.copy_from_slice(&data[..]);

        self.load_rom(data);
    }

    pub fn get_interrupt(&self) -> Option<(Interrupt, u16)> {
        let _ie = self.read_from_bus(HwRegister::IE as u16);
        let _if = self.read_from_bus(HwRegister::IF as u16);
        let interrupt_mask = _ie & _if;

        Interrupt::get_interrupt(interrupt_mask)
    }

    pub fn unset_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.read_from_bus(HwRegister::IF as u16);

        _if &= !(interrupt as u8);

        self.write_to_bus(HwRegister::IF as u16, _if);
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.read_from_bus(HwRegister::IF as u16);

        _if |= interrupt as u8;

        self.write_to_bus(HwRegister::IF as u16, _if);
    }

    pub fn inc_div(&mut self) {
        self.hw_registers._div = self.hw_registers._div.wrapping_add(1)
    }

    fn inc_tima(&mut self) {
        self.hw_registers._tima = self.hw_registers._tima.wrapping_add(1)
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
