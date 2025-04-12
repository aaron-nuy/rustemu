use crate::console::cartridge::Cartridge;
use crate::console::cpu::cpu::Cpu;
use crate::console::bus::Bus;
use crate::console::display::Display;
use crate::console::audio::Audio;
use std::fs;
use std::path::Path;
use crate::console::timer::Timer;

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    cartridge: Cartridge,
    display: Display,
    audio: Audio,
    timer: Timer,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cartridge: Cartridge::new(),
            display: Display::new(),
            audio: Audio::new(),
            timer: Timer::new()
        }
    }

    pub fn load(&mut self, cartridge_path: &str) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(cartridge_path);
        let data = fs::read(path).expect("Failed to read file");
    
        self.bus.load_rom(data);
    }

    pub fn run(&mut self) {
        loop {
            let instruction_c_cycles= (self.cpu.clock(&mut self.bus) as u64) * 4;
            self.timer.tick(instruction_c_cycles, &mut self.bus);
        }
    }
}