use crate::console::bus::Bus;
use crate::console::cpu::cpu::Cpu;
use crate::console::gui::gui::Gui;
use crate::console::hw_register::HwRegister;
use crate::console::timer::Timer;
use std::fs;
use std::path::Path;
use crate::console::constants::FRAME_DOT_CYCLES;

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

        assert_eq!(data.len(), crate::console::constants::CARTRIDGE_SIZE);
        self.bus.fill_cartridge(data);
    }

    pub fn run(&mut self) {
        let mut cycles_since_last_render = 0;
        let mut dot_cycles_to_run_cpu = 0;

        
        while !self.gui.should_close() {
            // Cpu ticks every 4 dot cycles
            if dot_cycles_to_run_cpu == 0 {
                dot_cycles_to_run_cpu = (self.cpu.tick(&mut self.bus) as u64) * 4;
            }

            self.timer.tick(&mut self.bus);
            self.bus.tick();

            if cycles_since_last_render >= FRAME_DOT_CYCLES {
                self.gui.update(self.bus.get_gpu_buffer()).unwrap();
                cycles_since_last_render = 0;
            }

            cycles_since_last_render += 1;
            dot_cycles_to_run_cpu -= 1;
        }
    }
}
