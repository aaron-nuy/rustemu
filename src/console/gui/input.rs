use minifb::Key;
use std::ops::Shl;

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

pub fn update_input(window: &mut minifb::Window, p1: &mut u8) {
    let dpad_selected = *p1 & P1Flags::DPAD as u8 == 0;
    let buttons_selected = *p1 & P1Flags::BUTTONS as u8 == 0;

    if !dpad_selected && !buttons_selected {
        *p1 |= 0x0F;
        return;
    }

    let a_pressed = window.is_key_down(A_BUTTON);
    let b_pressed = window.is_key_down(B_BUTTON);
    let select_pressed = window.is_key_down(SELECT_BUTTON);
    let start_pressed = window.is_key_down(START_BUTTON);
    let right_pressed = window.is_key_down(RIGHT_BUTTON);
    let left_pressed = window.is_key_down(LEFT_BUTTON);
    let up_pressed = window.is_key_down(UP_BUTTON);
    let down_pressed = window.is_key_down(DOWN_BUTTON);

    let first_bit = ((a_pressed && buttons_selected) || (right_pressed && dpad_selected)) as u8;
    let second_bit: u8 =
        (((b_pressed && buttons_selected) || (left_pressed && dpad_selected)) as u8).shl(1);
    let third_bit: u8 =
        (((select_pressed && buttons_selected) || (up_pressed && dpad_selected)) as u8).shl(2);
    let fourth_bit: u8 =
        (((start_pressed && buttons_selected) || (down_pressed && dpad_selected)) as u8).shl(3);

    *p1 |= 0x0F;
    *p1 &= !first_bit & !second_bit & !third_bit &!fourth_bit;
}
