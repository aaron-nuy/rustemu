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
    pub _p1: u8,
    pub _sb: u8,
    pub _sc: u8,
    pub _div: u8,
    pub _tima: u8,
    pub _tma: u8,
    pub _tac: u8,
    pub _if: u8,
    pub _nr10: u8,
    pub _nr11: u8,
    pub _nr12: u8,
    pub _nr13: u8,
    pub _nr14: u8,
    pub _nr21: u8,
    pub _nr22: u8,
    pub _nr23: u8,
    pub _nr24: u8,
    pub _nr30: u8,
    pub _nr31: u8,
    pub _nr32: u8,
    pub _nr33: u8,
    pub _nr34: u8,
    pub _nr41: u8,
    pub _nr42: u8,
    pub _nr43: u8,
    pub _nr44: u8,
    pub _nr50: u8,
    pub _nr51: u8,
    pub _nr52: u8,
    pub _lcdc: u8,
    pub _stat: u8,
    pub _scy: u8,
    pub _scx: u8,
    pub _ly: u8,
    pub _lyc: u8,
    pub _dma: u8,
    pub _bgp: u8,
    pub _obp0: u8,
    pub _obp1: u8,
    pub _wy: u8,
    pub _wx: u8,
    pub _ie: u8,
}
