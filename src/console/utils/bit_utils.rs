pub fn modify_bit(value: u8, bit_position: u8, on: bool) -> u8 {
    if on {
        value | (1 << bit_position)
    } else {
        value & !(1 << bit_position)
    }
}

pub fn colors_to_argb(r: u8, g: u8, b: u8) -> u32 {
    ((0xFF as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
