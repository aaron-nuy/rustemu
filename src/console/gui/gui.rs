use crate::console::constants::SCREEN_HEIGHT;
use crate::console::constants::SCREEN_WIDTH;
use crate::console::gui::display::*;
use crate::console::gui::input::*;
use crate::console::utils::bit_utils::colors_to_argb;
use minifb::{Key, Window, WindowOptions};

pub struct Gui {
    _window: Window,
    _display: Display,
    _input: Input,
}

impl Gui {
    pub fn new() -> Self {
        let window_options = WindowOptions {
            resize: false,
            title: true,
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        };
        let window = Window::new("rustemu", SCREEN_WIDTH, SCREEN_HEIGHT, window_options)
            .expect("Unable to open window");

        Self {
            _window: window,
            _display: Display::new(),
            _input: Input::new(),
        }
    }

    pub fn update(&mut self) -> minifb::Result<()> {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx: usize = SCREEN_WIDTH * y + x;

                if self._window.is_key_down(Key::Left) {
                    self._display.buffer[idx] =
                        colors_to_argb(y as u8, x as u8, ((x + y) / 2) as u8);
                }

                self._display.buffer[idx] = colors_to_argb(y as u8, x as u8, ((x + y) / 2) as u8);
            }
        }

        self._window
            .update_with_buffer(self._display.buffer.as_ref(), SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    pub fn should_close(&self) -> bool {
        !self._window.is_open()
    }
}
