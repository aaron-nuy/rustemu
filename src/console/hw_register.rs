use crate::console::constants::DMA_MULT;
use crate::console::dma::DMAData;
use crate::console::gui::gpu::{GpuMode, STATFlag};
use crate::console::gui::input::P1_WRITE_MASK;
use crate::console::hw_register::HwRegister::{DIV, IE, IF, LY, LYC, P1, STAT, TIMA};
use crate::console::interrupt::Interrupt;

const INNER_REG_ARR_SIZE: usize = 0x100;
const INNER_REG_IDX_FLAG: usize = 0x00FF;

#[repr(u16)]
#[derive(Copy, Clone)]
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
    #[inline]
    pub fn from_addr(addr: u16) -> HwRegister {
        debug_assert!(HwRegister::supported_addr(addr));
        unsafe { core::mem::transmute(addr) }
    }

    #[inline]
    pub fn supported_addr(addr: u16) -> bool {
        matches!(addr,
        0xff00..=0xff02 | 0xff04..=0xff07 | 0xff0f |
        0xff10..=0xff14 | 0xff16..=0xff19 | 0xff1a..=0xff1e |
        0xff20..=0xff26 | 0xff40..=0xff4b | 0xffff)
    }

    #[inline]
    fn to_index(self) -> usize {
        (self as usize) & INNER_REG_IDX_FLAG
    }
}

pub struct HwRegisters {
    regs: [u8; INNER_REG_ARR_SIZE],

    prev_stat_line: bool,
    stat_line: bool,
    pub dma_data: DMAData,
    pub dpad_state: u8,
    pub button_state: u8,
    prev_dpad_state: u8,
    prev_button_state: u8,
}

impl Default for HwRegisters {
    fn default() -> Self {
        Self {
            regs: [0u8; INNER_REG_ARR_SIZE],
            prev_stat_line: false,
            stat_line: false,
            dma_data: DMAData::default(),
            dpad_state: 0x0F,
            button_state: 0x0F,
            prev_dpad_state: 0x0F,
            prev_button_state: 0x0F,
        }
    }
}

impl HwRegisters {
    #[inline]
    fn reg_as_ref(&self, reg: HwRegister) -> &u8 {
        &self.regs[reg.to_index()]
    }

    #[inline]
    fn reg_as_mut_ref(&mut self, reg: HwRegister) -> &mut u8 {
        &mut self.regs[reg.to_index()]
    }

    #[inline]
    fn raw_read(&self, hw_register: HwRegister) -> u8 {
        unsafe { *self.regs.get_unchecked(hw_register.to_index()) }
    }

    #[inline]
    fn raw_write(&mut self, hw_register: HwRegister, value: u8) {
        unsafe { *self.regs.get_unchecked_mut(hw_register.to_index()) = value; }
    }

    pub fn write_to_register(&mut self, hw_register: HwRegister, value: u8) {
        use HwRegister::*;
        match hw_register {
            P1 => {
                let p1 = self.reg_as_mut_ref(P1);
                *p1 = (*p1 & !P1_WRITE_MASK) | (value & P1_WRITE_MASK);
            }
            SC => {
                let sc = if (value & 0x80) != 0 {
                    value & 0x7F
                } else {
                    value
                };
                self.raw_write(SC, sc);
            }
            DIV => self.raw_write(DIV, 0x00),
            TAC => {
                self.inc_tima();
                self.raw_write(TAC, value);
            }
            DMA => {
                self.raw_write(DMA, value);
                if !self.dma_data.running {
                    self.dma_data.init_transfer_data(value as u16 * DMA_MULT);
                }
            }
            _ => self.raw_write(hw_register, value),
        }
    }

    #[inline]
    pub fn write_to_register_addr(&mut self, addr: u16, value: u8) {
        self.write_to_register(HwRegister::from_addr(addr), value);
    }

    pub fn read_from_register(&self, hw_register: HwRegister) -> u8 {
        match hw_register {
            P1 => {
                use crate::console::gui::input::*;
                let p1 = self.raw_read(P1);
                let mut low_nibble = 0x0F;
                if p1 & (P1Flags::DPAD as u8) == 0 {
                    low_nibble &= self.dpad_state;
                }
                if p1 & (P1Flags::BUTTONS as u8) == 0 {
                    low_nibble &= self.button_state;
                }
                (p1 & 0xF0) | low_nibble
            }
            _ => self.raw_read(hw_register),
        }
    }

    #[inline]
    pub fn read_from_register_addr(&self, addr: u16) -> u8 {
        self.read_from_register(HwRegister::from_addr(addr))
    }

    #[inline]
    pub fn inc_div(&mut self) {
        let div_ref = self.reg_as_mut_ref(DIV);
        *div_ref = div_ref.wrapping_add(1);
    }

    #[inline]
    pub fn inc_tima(&mut self) {
        let tima_ref = self.reg_as_mut_ref(TIMA);
        *tima_ref = tima_ref.wrapping_add(1);
    }

    #[inline]
    pub fn get_interrupt(&self) -> Option<(Interrupt, u16)> {
        Interrupt::get_interrupt(self.raw_read(IE) & self.raw_read(IF))
    }

    #[inline]
    pub fn unset_interrupt(&mut self, interrupt: Interrupt) {
        *self.reg_as_mut_ref(IF) &= !(interrupt as u8);
    }

    #[inline]
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        *self.reg_as_mut_ref(IF) |= interrupt as u8;
    }

    pub fn handle_lyc_cond(&mut self) {
        if self.raw_read(LY) == self.raw_read(LYC) {
            let stat_ref = self.reg_as_mut_ref(STAT);
            *stat_ref |= STATFlag::LYEqLYC as u8;
            if *stat_ref & (STATFlag::LYCIntSelect as u8) != 0 {
                self.stat_line = true;
            }
        } else {
            let stat_ref = self.reg_as_mut_ref(STAT);
            *stat_ref &= !(STATFlag::LYEqLYC as u8);
        }
    }

    #[inline]
    pub fn set_stat_gpu_mode(&mut self, gpu_mode: GpuMode) {
        let mask = STATFlag::PPUMode as u8;
        let stat_ref = self.reg_as_mut_ref(STAT);
        *stat_ref = (*stat_ref & !mask) | (gpu_mode as u8);
    }

    #[inline]
    pub fn update_stat_line(&mut self) {
        self.prev_stat_line = self.stat_line;
        self.stat_line = false;
    }

    #[inline]
    pub fn handle_stat_line(&mut self) {
        if !self.prev_stat_line && self.stat_line {
            self.request_interrupt(Interrupt::STAT);
        }
    }

    #[inline]
    pub fn handle_stat_line_mode_cond(&mut self, flag: STATFlag) {
        if self.raw_read(STAT) & (flag as u8) != 0 {
            self.stat_line = true;
        }
    }

    #[inline]
    pub fn handle_stat_line_mode0_cond(&mut self) {
        self.handle_stat_line_mode_cond(STATFlag::Mode0IntSelect);
    }

    #[inline]
    pub fn handle_stat_line_mode1_cond(&mut self) {
        self.handle_stat_line_mode_cond(STATFlag::Mode1IntSelect);
    }

    #[inline]
    pub fn handle_stat_line_mode2_cond(&mut self) {
        self.handle_stat_line_mode_cond(STATFlag::Mode2IntSelect);
    }

    pub fn update_input_state(&mut self, dpad_state: u8, button_state: u8) {
        self.prev_dpad_state = self.dpad_state;
        self.prev_button_state = self.button_state;

        self.dpad_state = dpad_state & 0x0F;
        self.button_state = button_state & 0x0F;

        let dpad_fell = !self.dpad_state & self.prev_dpad_state;
        let button_fell = !self.button_state & self.prev_button_state;

        if (dpad_fell | button_fell) != 0 {
            self.request_interrupt(Interrupt::Joypad);
        }
    }
}
