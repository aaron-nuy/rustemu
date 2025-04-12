use crate::console::constants::*;
use crate::console::hw_register::{HwRegisterAddr, HwRegisters};
use crate::console::interrupt::Interrupt;

pub struct Bus {
    _ram: Box<[u8; MEMORY_SIZE]>,
    hw_registers: HwRegisters,
}

impl Bus {
    fn write_to_bus(&mut self, addr: u16, value: u8) {
        match addr {
            addr if HwRegisters::supported_addr(addr) => self.hw_registers.set_addr(addr, value),
            _ => self._ram[addr as usize] = value,
        }
    }

    fn read_from_bus(&self, addr: u16) -> u8 {
        match addr {
            addr if HwRegisters::supported_addr(addr) => self.hw_registers.get_addr(addr),
            _ => self._ram[addr as usize],
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
        self._ram[..data.len()].copy_from_slice(&data);
    }

    pub fn get_interrupt(&self) -> Option<(Interrupt, u16)> {
        let _ie = self.hw_registers.get_hw_register(HwRegisterAddr::IE);
        let _if = self.hw_registers.get_hw_register(HwRegisterAddr::IF);
        let interrupt_mask = _ie & _if;

        Interrupt::get_interrupt(interrupt_mask)
    }

    pub fn new() -> Self {
        Self {
            _ram: Box::new([0; MEMORY_SIZE]),
            hw_registers: HwRegisters::default(),
        }
    }
}
