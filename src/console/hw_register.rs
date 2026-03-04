use crate::console::constants::DMA_MULT;
use crate::console::dma::DMAData;
use crate::console::gui::gpu::{GpuMode, STATFlag};
use crate::console::interrupt::Interrupt;
use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(Copy, Clone, TryFromPrimitive)]
pub enum HwRegister {
    P1 = 0xff00,
    SB = 0xff01,
    SC = 0xff02,
    DIV = 0xff04,
    TIMA = 0xff05,
    TMA = 0xff06,
    TAC = 0xff07,
    IF = 0xff0f,
    NR10 = 0xff10,
    NR11 = 0xff11,
    NR12 = 0xff12,
    NR13 = 0xff13,
    NR14 = 0xff14,
    NR21 = 0xff16,
    NR22 = 0xff17,
    NR23 = 0xff18,
    NR24 = 0xff19,
    NR30 = 0xff1a,
    NR31 = 0xff1b,
    NR32 = 0xff1c,
    NR33 = 0xff1d,
    NR34 = 0xff1e,
    NR41 = 0xff20,
    NR42 = 0xff21,
    NR43 = 0xff22,
    NR44 = 0xff23,
    NR50 = 0xff24,
    NR51 = 0xff25,
    NR52 = 0xff26,
    LCDC = 0xff40,
    STAT = 0xff41,
    SCY = 0xff42,
    SCX = 0xff43,
    LY = 0xff44,
    LYC = 0xff45,
    DMA = 0xff46,
    BGP = 0xff47,
    OBP0 = 0xff48,
    OBP1 = 0xff49,
    WY = 0xff4a,
    WX = 0xff4b,
    IE = 0xffff,
}

impl HwRegister {
    pub fn from_addr(addr: u16) -> HwRegister {
        HwRegister::try_from(addr).unwrap()
    }

    pub fn supported_addr(addr: u16) -> bool {
        match HwRegister::try_from(addr) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[derive(Default)]
pub struct HwRegisters {
    _p1: u8,
    _sb: u8,
    _sc: u8,
    _div: u8,
    _tima: u8,
    _tma: u8,
    _tac: u8,
    _if: u8,
    _nr10: u8,
    _nr11: u8,
    _nr12: u8,
    _nr13: u8,
    _nr14: u8,
    _nr21: u8,
    _nr22: u8,
    _nr23: u8,
    _nr24: u8,
    _nr30: u8,
    _nr31: u8,
    _nr32: u8,
    _nr33: u8,
    _nr34: u8,
    _nr41: u8,
    _nr42: u8,
    _nr43: u8,
    _nr44: u8,
    _nr50: u8,
    _nr51: u8,
    _nr52: u8,
    _lcdc: u8,
    _stat: u8,
    _scy: u8,
    _scx: u8,
    _ly: u8,
    _lyc: u8,
    _dma: u8,
    _bgp: u8,
    _obp0: u8,
    _obp1: u8,
    _wy: u8,
    _wx: u8,
    _ie: u8,
    prev_stat_line: bool,
    stat_line: bool,
    pub dma_data: DMAData,
    pub dpad_state: u8,
    pub button_state: u8,
    prev_dpad_state: u8,
    prev_button_state: u8,
}

impl HwRegisters {
    pub fn write_to_register(&mut self, hw_register: HwRegister, value: u8) {
        use HwRegister::*;
        match hw_register {
            P1 => {
                use crate::console::gui::input::*;

                self._p1 = (self._p1 & !P1_WRITE_MASK) | (value & P1_WRITE_MASK);
            }
            SB => self._sb = value,
            SC => {
                self._sc = value;

                if (value & 0x80) != 0 {
                    //print!("{}", self._sb as char);
                    self._sc &= 0x7F;
                }
            }
            DIV => self._div = 0x00,
            TIMA => self._tima = value,
            TMA => self._tma = value,
            TAC => {
                self.inc_tima();
                self._tac = value
            }
            IF => self._if = value,
            NR10 => self._nr10 = value,
            NR11 => self._nr11 = value,
            NR12 => self._nr12 = value,
            NR13 => self._nr13 = value,
            NR14 => self._nr14 = value,
            NR21 => self._nr21 = value,
            NR22 => self._nr22 = value,
            NR23 => self._nr23 = value,
            NR24 => self._nr24 = value,
            NR30 => self._nr30 = value,
            NR31 => self._nr31 = value,
            NR32 => self._nr32 = value,
            NR33 => self._nr33 = value,
            NR34 => self._nr34 = value,
            NR41 => self._nr41 = value,
            NR42 => self._nr42 = value,
            NR43 => self._nr43 = value,
            NR44 => self._nr44 = value,
            NR50 => self._nr50 = value,
            NR51 => self._nr51 = value,
            NR52 => self._nr52 = value,
            LCDC => self._lcdc = value,
            STAT => {
                // TODO: Trigger interrupt only on rising edge of stat line?
                self._stat = value;
            }
            SCY => self._scy = value,
            SCX => self._scx = value,
            LY => self._ly = value, // TODO: Make it read only for cpu
            LYC => self._lyc = value,
            DMA => {
                self._dma = value;
                if (!self.dma_data.running) {
                    self.dma_data
                        .init_transfer_data(self._dma as u16 * DMA_MULT);
                }
            }
            BGP => self._bgp = value,
            OBP0 => self._obp0 = value,
            OBP1 => self._obp1 = value,
            WY => self._wy = value,
            WX => self._wx = value,
            IE => self._ie = value,
        }
    }

    pub fn write_to_register_addr(&mut self, addr: u16, value: u8) {
        let hw_register_addr = HwRegister::from_addr(addr);
        self.write_to_register(hw_register_addr, value);
    }

    pub fn read_from_register(&self, hw_register: HwRegister) -> u8 {
        use HwRegister::*;
        match hw_register {
            P1 => {
                use crate::console::gui::input::*;
                let mut low_nibble = 0x0F;
                if self._p1 & (P1Flags::DPAD as u8) == 0 {
                    low_nibble &= self.dpad_state;
                }
                if self._p1 & (P1Flags::BUTTONS as u8) == 0 {
                    low_nibble &= self.button_state;
                }

                (self._p1 & 0xF0) | low_nibble
            }
            SB => self._sb,
            SC => self._sc,
            DIV => self._div,
            TIMA => self._tima,
            TMA => self._tma,
            TAC => self._tac,
            IF => self._if,
            NR10 => self._nr10,
            NR11 => self._nr11,
            NR12 => self._nr12,
            NR13 => self._nr13,
            NR14 => self._nr14,
            NR21 => self._nr21,
            NR22 => self._nr22,
            NR23 => self._nr23,
            NR24 => self._nr24,
            NR30 => self._nr30,
            NR31 => self._nr31,
            NR32 => self._nr32,
            NR33 => self._nr33,
            NR34 => self._nr34,
            NR41 => self._nr41,
            NR42 => self._nr42,
            NR43 => self._nr43,
            NR44 => self._nr44,
            NR50 => self._nr50,
            NR51 => self._nr51,
            NR52 => self._nr52,
            LCDC => self._lcdc,
            STAT => self._stat,
            SCY => self._scy,
            SCX => self._scx,
            LY => self._ly,
            LYC => self._lyc,
            DMA => self._dma,
            BGP => self._bgp,
            OBP0 => self._obp0,
            OBP1 => self._obp1,
            WY => self._wy,
            WX => self._wx,
            IE => self._ie,
        }
    }

    pub fn read_from_register_addr(&self, hw_register_addr: u16) -> u8 {
        let hw_register = HwRegister::from_addr(hw_register_addr);
        self.read_from_register(hw_register)
    }

    pub fn inc_div(&mut self) {
        self._div = self._div.wrapping_add(1)
    }

    pub fn inc_tima(&mut self) {
        self._tima = self._tima.wrapping_add(1)
    }

    pub fn get_interrupt(&self) -> Option<(Interrupt, u16)> {
        let _ie = self.read_from_register(HwRegister::IE);
        let _if = self.read_from_register(HwRegister::IF);
        let interrupt_mask = _ie & _if;

        Interrupt::get_interrupt(interrupt_mask)
    }

    pub fn unset_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.read_from_register(HwRegister::IF);

        _if &= !(interrupt as u8);

        self.write_to_register(HwRegister::IF, _if);
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        let mut _if = self.read_from_register(HwRegister::IF);

        _if |= interrupt as u8;

        self.write_to_register(HwRegister::IF, _if);
    }

    pub fn handle_lyc_cond(&mut self) {
        if self._ly == self._lyc {
            self._stat |= STATFlag::LYEqLYC as u8;

            if self._stat & (STATFlag::LYCIntSelect as u8) != 0 {
                self.stat_line |= true;
            }
        } else {
            self._stat &= !(STATFlag::LYEqLYC as u8);
        }
    }

    pub fn set_stat_gpu_mode(&mut self, gpu_mode: GpuMode) {
        let mask = STATFlag::PPUMode as u8;
        let gpu_mode = gpu_mode as u8;
        self._stat = (self._stat & !mask) | (gpu_mode & mask);
    }

    pub fn update_stat_line(&mut self) {
        self.prev_stat_line = self.stat_line;
        self.stat_line = false;
    }

    pub fn handle_stat_line(&mut self) {
        if !self.prev_stat_line && self.stat_line {
            self.request_interrupt(Interrupt::STAT);
        }
    }

    pub fn handle_stat_line_mode0_cond(&mut self) {
        if self._stat & (STATFlag::Mode0IntSelect as u8) != 0 {
            self.stat_line |= true;
        }
    }

    pub fn handle_stat_line_mode1_cond(&mut self) {
        if self._stat & (STATFlag::Mode1IntSelect as u8) != 0 {
            self.stat_line |= true;
        }
    }

    pub fn handle_stat_line_mode2_cond(&mut self) {
        if self._stat & (STATFlag::Mode2IntSelect as u8) != 0 {
            self.stat_line |= true;
        }
    }

    pub fn update_input_state(&mut self, dpad_state: u8, button_state: u8) {
        self.prev_dpad_state = self.dpad_state;
        self.prev_button_state = self.button_state;
        self.dpad_state = dpad_state;
        self.button_state = button_state;

        let dpad_fell = !self.dpad_state & self.prev_dpad_state;
        let button_fell = !self.button_state & self.prev_button_state;

        if (dpad_fell | button_fell) & 0x0F != 0 {
            self.request_interrupt(Interrupt::Joypad);
        }
    }
}
