#[derive(Copy, Clone)]
pub enum HwRegisterAddr {
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

impl HwRegisterAddr {
    pub fn from_addr(addr: u16) -> HwRegisterAddr {
        use HwRegisterAddr::*;
        match addr {
            0xff00 => P1,
            0xff01 => SB,
            0xff02 => SC,
            0xff04 => DIV,
            0xff05 => TIMA,
            0xff06 => TMA,
            0xff07 => TAC,
            0xff0f => IF,
            0xff10 => NR10,
            0xff11 => NR11,
            0xff12 => NR12,
            0xff13 => NR13,
            0xff14 => NR14,
            0xff16 => NR21,
            0xff17 => NR22,
            0xff18 => NR23,
            0xff19 => NR24,
            0xff1a => NR30,
            0xff1b => NR31,
            0xff1c => NR32,
            0xff1d => NR33,
            0xff1e => NR34,
            0xff20 => NR41,
            0xff21 => NR42,
            0xff22 => NR43,
            0xff23 => NR44,
            0xff24 => NR50,
            0xff25 => NR51,
            0xff26 => NR52,
            0xff40 => LCDC,
            0xff41 => STAT,
            0xff42 => SCY,
            0xff43 => SCX,
            0xff44 => LY,
            0xff45 => LYC,
            0xff46 => DMA,
            0xff47 => BGP,
            0xff48 => OBP0,
            0xff49 => OBP1,
            0xff4a => WY,
            0xff4b => WX,
            0xffff => IE,
            _ => panic!("Unsupported hardware register address: 0x{:X}", addr),
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
}

impl HwRegisters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_hw_register(&mut self, hw_register_addr: HwRegisterAddr, value: u8) {
        use HwRegisterAddr::*;
        match hw_register_addr {
            P1 => self._p1 = value,
            SB => self._sb = value,
            SC => {
                self._sc = value;

                if (value & 0x80) != 0 {
                    print!("{}", self._sb as char);
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
            STAT => self._stat = value,
            SCY => self._scy = value,
            SCX => self._scx = value,
            LY => self._ly = value,
            LYC => self._lyc = value,
            DMA => self._dma = value,
            BGP => self._bgp = value,
            OBP0 => self._obp0 = value,
            OBP1 => self._obp1 = value,
            WY => self._wy = value,
            WX => self._wx = value,
            IE => self._ie = value,
        }
    }

    pub fn get_hw_register(&self, hw_register_addr: HwRegisterAddr) -> u8 {
        use HwRegisterAddr::*;
        match hw_register_addr {
            P1 => self._p1,
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

    pub fn set_addr(&mut self, addr: u16, value: u8) {
        let hw_register_addr = HwRegisterAddr::from_addr(addr);
        self.set_hw_register(hw_register_addr, value)
    }

    pub fn get_addr(&self, addr: u16) -> u8 {
        let hw_register_addr = HwRegisterAddr::from_addr(addr);
        self.get_hw_register(hw_register_addr)
    }

    pub fn inc_div(&mut self) {
        self._div = self._div.wrapping_add(1)
    }

    fn inc_tima(&mut self) {
        self._tima = self._tima.wrapping_add(1)
    }

    pub fn supported_addr(addr: u16) -> bool {
        match addr {
            0xff00 |
            0xff01 |
            0xff02 |
            0xff04 |
            0xff05 |
            0xff06 |
            0xff07 |
            0xff0f |
            0xff10 |
            0xff11 |
            0xff12 |
            0xff13 |
            0xff14 |
            0xff16 |
            0xff17 |
            0xff18 |
            0xff19 |
            0xff1a |
            0xff1b |
            0xff1c |
            0xff1d |
            0xff1e |
            0xff20 |
            0xff21 |
            0xff22 |
            0xff23 |
            0xff24 |
            0xff25 |
            0xff26 |
            0xff40 |
            0xff41 |
            0xff42 |
            0xff43 |
            0xff44 |
            0xff45 |
            0xff46 |
            0xff47 |
            0xff48 |
            0xff49 |
            0xff4a |
            0xff4b |
            0xffff => true,
            _ => false,
        }
    }
}