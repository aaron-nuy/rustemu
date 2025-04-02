use crate::console::cartridge::Cartridge;
use crate::console::cpu::cpu::Cpu;
use crate::console::memory::Memory;
use crate::console::display::Display;
use crate::console::audio::Audio;

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory,
    cartridge: Cartridge,
    display: Display,
    audio: Audio
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
            cartridge: Cartridge::new(),
            display: Display::new(),
            audio: Audio::new()
        }
    } 
}