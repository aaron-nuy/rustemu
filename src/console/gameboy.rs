use crate::console::bus::Bus;
use crate::console::cpu::cpu::Cpu;
use crate::console::gui::gui::Gui;
use crate::console::timer::Timer;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use crate::console::hw_register::{HwRegisterAddr, HwRegisters};
use crate::console::interrupt::Interrupt::VBlank;

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    timer: Timer,
    gui: Gui,
}

impl Gameboy {
    pub fn new() -> Self {
        let m_gui = Gui::new();
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            timer: Timer::new(),
            gui: m_gui,
        }
    }

    pub fn load(&mut self, cartridge_path: &str) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(cartridge_path);
        let data = fs::read(path).expect("Failed to read file");

        self.bus.load_rom(data);
    }

    pub fn run(&mut self) {
        let mut cycles_since_last_render = 0;
        const RENDER_FREQUENCY: u64 = 70224;
        loop {
            let instruction_c_cycles = (self.cpu.clock(&mut self.bus) as u64) * 4;
            self.timer.tick(instruction_c_cycles, &mut self.bus);

            cycles_since_last_render += instruction_c_cycles;

            if cycles_since_last_render >= RENDER_FREQUENCY / 64 {
                self.bus.write_to_8b(HwRegisterAddr::LY.to_addr(), 144);
                let gpu_out = self.bus.gpu.tick(instruction_c_cycles, &self.bus);
                self.gui.update(&gpu_out).unwrap();
                cycles_since_last_render = 0;
            }

            if self.gui.should_close() {
                break;
            }

        }
    }
}
