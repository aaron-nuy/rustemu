use crate::console::bus::Bus;
use crate::console::cpu::cpu::Cpu;
use crate::console::gui::gui::{Gui, Palette};
use crate::console::timer::Timer;
use crate::read_rom;
use core::time::Duration;
#[cfg(not(efi))]
use std::time::Instant;

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

    pub fn new_with_pal(z: u32, o: u32, t: u32, tr: u32) -> Self {
        let palette = Palette::new(z, o, t, tr);
        let m_gui = Gui::new_with_pal(palette);
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            timer: Timer::new(),
            gui: m_gui,
        }
    }

    pub fn load(&mut self, cartridge_path: &str) {
        let data = read_rom::read_file(cartridge_path);
        self.bus.load_rom(&data);
    }

    pub fn run(&mut self) {
        let mut dot_cycles_to_run_cpu = 0;

        const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);
        #[cfg(not(efi))]
        let mut frame_start = Instant::now();
        while !self.gui.should_close() {
            // Cpu ticks every 4 dot cycles
            if dot_cycles_to_run_cpu == 0 {
                dot_cycles_to_run_cpu = (self.cpu.tick(&mut self.bus) as u64) * 4;
            }

            self.timer.tick(&mut self.bus);
            self.bus.tick();

            dot_cycles_to_run_cpu -= 1;

            cfg_if::cfg_if! {
                if #[cfg(efi)] {
                    if self.bus.is_vblank_start() {
                        use uefi::prelude::*;
                        self.gui.update(&mut self.bus);
                        boot::stall(FRAME_DURATION.as_micros() as usize);
                    }
                } else {
                    if self.bus.is_vblank_start() {
                        self.gui.update(&mut self.bus);
                        let elapsed = frame_start.elapsed();
                        if elapsed < FRAME_DURATION {
                            std::thread::sleep(FRAME_DURATION - elapsed);
                        }
                        frame_start = Instant::now();
                    }
                }
            }
        }
    }
}
