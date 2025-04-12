#[derive(Clone, Copy)]
pub enum Interrupt {
    VBlank = 0x1,
    STAT = 0x02,
    Timer = 0x04,
    Serial = 0x08,
    Joypad = 0x10,
}

impl Interrupt {

    pub fn get_interrupt(mask: u8) -> Option<(Interrupt, u16)> {
        use Interrupt::*;
        match mask {
            mask if (mask & VBlank as u8) != 0 => Some((VBlank, 0x40)),
            mask if (mask & STAT as u8) != 0 => Some((STAT, 0x48)),
            mask if (mask & Timer as u8) != 0 => Some((Timer, 0x50)),
            mask if (mask & Serial as u8) != 0 => Some((Serial, 0x58)),
            mask if (mask & Joypad as u8) != 0 => Some((Joypad, 0x60)),
            _ => None,
        }
    }

}