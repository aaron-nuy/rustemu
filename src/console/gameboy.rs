use crate::console::cartridge::Cartridge;
use crate::console::cpu::cpu::Cpu;
use crate::console::bus::Bus;
use crate::console::display::Display;
use crate::console::audio::Audio;
use std::fs;
use std::path::Path;

use super::{cpu, bus};

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    cartridge: Cartridge,
    display: Display,
    audio: Audio
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cartridge: Cartridge::new(),
            display: Display::new(),
            audio: Audio::new()
        }
    }

    pub fn load(&mut self, cartridge_path: &str) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(cartridge_path);
        let data = fs::read(path).expect("Failed to read file");
    
        self.bus.load_rom(data);
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.clock(&mut self.bus);
        }
    }
}