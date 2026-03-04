use crate::console::audio::Audio;
use crate::console::cartridge::Cartridge;
use crate::console::constants::*;
use crate::console::gui::gpu::{Gpu, PixelLevel};
use crate::console::hw_register::HwRegister;
use crate::console::hw_register::HwRegisters;
use crate::console::interrupt::Interrupt;

pub struct Bus {
    ram: Box<[u8; MEMORY_SIZE]>,
    boot_rom: [u8; BOOT_ROM_SIZE],
    boot_rom_enabled: bool,
    cartridge: Cartridge,
    gpu: Gpu,
    audio: Audio,
    hw_registers: HwRegisters,
}

impl Bus {
    fn write_to_bus(&mut self, addr: u16, value: u8) {
        match addr {
            addr if addr == BOOT_ROM_DISABLE_ADDR => {
                self.boot_rom_enabled = false;
            }
            VRAM_BEGIN..=VRAM_END => {
                self.gpu.write_to_vram(addr - VRAM_BEGIN, value);
            }
            addr if HwRegister::supported_addr(addr) => {
                self.hw_registers.write_to_register_addr(addr, value)
            }
            _ => self.ram[addr as usize] = value,
        }
    }

    fn read_from_bus(&self, addr: u16) -> u8 {
        match addr {
            addr if self.boot_rom_enabled && addr < BOOT_ROM_SIZE as u16 => {
                self.boot_rom[addr as usize]
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.read_from_vram(addr - VRAM_BEGIN),
            addr if HwRegister::supported_addr(addr) => {
                self.hw_registers.read_from_register_addr(addr)
            }
            _ => self.ram[addr as usize],
        }
    }

    //TODO: Maybe perform checks on who and when is attempting to access the bus and prevent it
    //      if it shouldn't be accessing it during this period
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
        self.hw_registers.get_interrupt()
    }

    pub fn unset_interrupt(&mut self, interrupt: Interrupt) {
        self.hw_registers.unset_interrupt(interrupt);
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.hw_registers.request_interrupt(interrupt);
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

    pub fn get_gpu_buffer(&self) -> &[PixelLevel; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.gpu.buffer
    }

    pub fn tick(&mut self) {
        if self.hw_registers.dma_data.running {
            // Put at top to ensure one machine cycle delay
            self.hw_registers.dma_data.dot_cycle_since_start += 1;

            // transfers one byte every 4 dot cycles
            if self.hw_registers.dma_data.dot_cycle_since_start % 4 == 0 {
                let val_to_write = self.read_from_bus(self.hw_registers.dma_data.current_addr);

                let offset = self
                    .hw_registers
                    .dma_data
                    .current_addr
                    .wrapping_sub(self.hw_registers.dma_data.start_addr);
                let dest_addr = offset.wrapping_add(OAM_BEGIN);

                self.write_to_bus(dest_addr, val_to_write);

                if self.hw_registers.dma_data.current_addr
                    == self.hw_registers.dma_data.start_addr + OAM_SIZE - 1
                {
                    self.hw_registers.dma_data.running = false;
                }

                self.hw_registers.dma_data.current_addr =
                    self.hw_registers.dma_data.current_addr.wrapping_add(1);
            }
        }

        self.hw_registers.update_stat_line();

        self.gpu.tick(&mut self.hw_registers);

        self.hw_registers.set_stat_gpu_mode(self.gpu.gpu_mode);

        self.hw_registers.handle_lyc_cond();

        self.hw_registers.handle_stat_line();
    }

    pub fn update_input_state(&mut self, dpad_state: u8, button_state: u8) {
        self.hw_registers
            .update_input_state(dpad_state, button_state);
    }
}
