pub fn modify_bit(value: u8, bit_position: u8, on: bool) -> u8 {
    if on { value | (1 << bit_position) } else { value & !(1 << bit_position) }
}
