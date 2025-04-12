use crate::console::bus::Bus;
use crate::console::hw_register::HwRegisterAddr;
use crate::console::constants::*;
use crate::console::interrupt::Interrupt;

#[derive(Default)]
pub struct Timer {
    _c_cycles: u64,
    _c_cycles_since_div: u64,
    _c_cycles_since_tima: u64,
}

impl Timer {

    pub fn new() -> Self {
        Self::default()
    }

    fn inc_div(&mut self, bus: &mut Bus) {
        while self._c_cycles_since_div >= TIMER_DIV_INC_RATE {
            bus.inc_div();
            self._c_cycles_since_div -= TIMER_DIV_INC_RATE;
        }
    }

    fn inc_tima(&mut self, bus: &mut Bus) {
        let tac = bus.read_from_8b(HwRegisterAddr::TAC as u16);

        if (tac & 0b100) == 0 {
            return;
        }

        let tac_cycle = Self::cycle_from_tac(tac);
        let tma = bus.read_from_8b(HwRegisterAddr::TMA as u16);

        while self._c_cycles_since_tima >= tac_cycle {
            self._c_cycles_since_tima -= tac_cycle;

            let current_tima = bus.read_from_8b(HwRegisterAddr::TIMA as u16);
            let (new_tima, overflow) = current_tima.overflowing_add(1);

            if overflow {
                bus.write_to_8b(HwRegisterAddr::TIMA as u16, tma);
                bus.trigger_interrupt(Interrupt::Timer);
            }
            else {
                bus.write_to_8b(HwRegisterAddr::TIMA as u16, new_tima);
            }
        }
    }

    pub fn tick(&mut self, added_cycles: u64, bus: &mut Bus) {
        self._c_cycles = self._c_cycles.wrapping_add(added_cycles);
        self._c_cycles_since_tima = self._c_cycles_since_tima.wrapping_add(added_cycles);
        self._c_cycles_since_div = self._c_cycles_since_div.wrapping_add(added_cycles);

        self.inc_tima(bus);
        self.inc_div(bus);
    }

    fn cycle_from_tac(tac: u8) -> u64 {
        let tac = tac & 0b11;

        match tac {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => unreachable!(),
        }
    }

}