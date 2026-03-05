pub enum P1Flags {
    DPAD = 0b0001_0000,
    BUTTONS = 0b0010_0000,
}
pub const P1_WRITE_MASK: u8 = 0b1111_0000;

#[cfg(not(efi))]
pub fn compute_input_states(window: &minifb::Window) -> (u8, u8) {
    use minifb::Key;

    pub const A_BUTTON: Key = Key::A;
    pub const B_BUTTON: Key = Key::B;
    pub const START_BUTTON: Key = Key::Space;
    pub const SELECT_BUTTON: Key = Key::E;
    pub const UP_BUTTON: Key = Key::Up;
    pub const DOWN_BUTTON: Key = Key::Down;
    pub const LEFT_BUTTON: Key = Key::Left;
    pub const RIGHT_BUTTON: Key = Key::Right;

    let right = (!window.is_key_down(RIGHT_BUTTON)) as u8;
    let left = (!window.is_key_down(LEFT_BUTTON)) as u8;
    let up = (!window.is_key_down(UP_BUTTON)) as u8;
    let down = (!window.is_key_down(DOWN_BUTTON)) as u8;
    let a = (!window.is_key_down(A_BUTTON)) as u8;
    let b = (!window.is_key_down(B_BUTTON)) as u8;
    let select = (!window.is_key_down(SELECT_BUTTON)) as u8;
    let start = (!window.is_key_down(START_BUTTON)) as u8;

    let dpad = right | (left << 1) | (up << 2) | (down << 3);
    let buttons = a | (b << 1) | (select << 2) | (start << 3);
    (dpad, buttons)
}

#[cfg(efi)]
pub fn compute_input_states() -> (u8, u8) {
    use uefi::boot;
    use uefi::proto::console::text::*;

    let mut right = true;
    let mut left = true;
    let mut up = true;
    let mut down = true;
    let mut a = true;
    let mut b = true;
    let mut select = true;
    let mut start = true;

    let handle = boot::get_handle_for_protocol::<Input>();
    if handle.is_err() {
        return (0x0F, 0x0F);
    }

    let input = boot::open_protocol_exclusive::<Input>(handle.unwrap());
    if input.is_err() {
        return (0x0F, 0x0F);
    }

    let mut input = input.unwrap();

    loop {
        let key = input.read_key();
        if key.is_err() {
            break;
        }
        let key = key.unwrap();
        if key.is_none() {
            break;
        }
        let key = key.unwrap();

        // false means pressed
        if key == Key::Special(ScanCode::UP) {
            up = false;
        }
        if key == Key::Special(ScanCode::DOWN) {
            down = false;
        }
        if key == Key::Special(ScanCode::LEFT) {
            left = false;
        }
        if key == Key::Special(ScanCode::RIGHT) {
            right = false;
        }

        if let Key::Printable(c) = key {
            let c = u16::from(c) as u8 as char;
            if c == 'a' || c == 'A' {
                a = false;
            }
            if c == 'b' || c == 'B' {
                b = false;
            }
            if c == 'e' || c == 'E' {
                select = false;
            }
            if c == ' ' {
                start = false;
            }
        }
    }

    let dpad = right as u8 | (left as u8) << 1 | (up as u8) << 2 | (down as u8) << 3;
    let buttons = a as u8 | (b as u8) << 1 | (select as u8) << 2 | (start as u8) << 3;
    (dpad, buttons)
}
