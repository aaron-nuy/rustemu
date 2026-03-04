use minifb::Key;

pub const A_BUTTON: Key = Key::A;
pub const B_BUTTON: Key = Key::B;
pub const START_BUTTON: Key = Key::Space;
pub const SELECT_BUTTON: Key = Key::E;
pub const UP_BUTTON: Key = Key::Up;
pub const DOWN_BUTTON: Key = Key::Down;
pub const LEFT_BUTTON: Key = Key::Left;
pub const RIGHT_BUTTON: Key = Key::Right;

pub const P1_WRITE_MASK: u8 = 0b1111_0000;

pub enum P1Flags {
    DPAD = 0b0001_0000,
    BUTTONS = 0b0010_0000,
}

pub fn compute_input_states(window: &minifb::Window) -> (u8, u8) {
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
