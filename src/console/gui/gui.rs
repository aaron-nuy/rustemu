use crate::console::constants::SCREEN_HEIGHT;
use crate::console::constants::SCREEN_WIDTH;
use crate::console::gui::gpu::PixelLevel;
use minifb::{Key, Window, WindowOptions};

pub struct Palette {
    zero: u32,
    one: u32,
    two: u32,
    three: u32,
}

impl Palette {
    pub fn default() -> Self {
        Self {
            zero: 0xE0F8D0,
            one: 0x88C070,
            two: 0x346856,
            three: 0x081820,
        }
    }

    pub fn new(z: u32, o: u32, t: u32, tr: u32) -> Self {
        Self {
            zero: z,
            one: o,
            two: t,
            three: tr,
        }
    }

    pub fn translate_palette(&mut self, pixel_level: PixelLevel) -> u32 {
        match (pixel_level) {
            PixelLevel::Zero => self.zero,
            PixelLevel::One => self.one,
            PixelLevel::Two => self.two,
            PixelLevel::Three => self.three,
        }
    }
}

pub struct Gui {
    palette: Palette,
    window: Window,
    display: [u32; SCREEN_WIDTH * SCREEN_HEIGHT]
}

impl Gui {
    pub fn new() -> Self {
        let window_options = WindowOptions {
            resize: false,
            title: true,
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        };
        let m_window = Window::new("rustemu", SCREEN_WIDTH, SCREEN_HEIGHT, window_options)
            .expect("Unable to open window");

        Self {
            palette: Palette::default(),
            window: m_window,
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT]
        }
    }

    pub fn new_with_pal(pal: Palette) -> Self {
        let window_options = WindowOptions {
            resize: false,
            title: true,
            scale: minifb::Scale::X4,
            ..WindowOptions::default()
        };
        let m_window = Window::new("rustemu", SCREEN_WIDTH, SCREEN_HEIGHT, window_options)
            .expect("Unable to open window");

        Self {
            palette: pal,
            window: m_window,
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT]
        }
    }

    pub fn update(
        &mut self,
        bus: &mut crate::console::bus::Bus,
    ) {
        let gpu_buffer = bus.get_gpu_buffer();

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx: usize = SCREEN_WIDTH * y + x;

                self.display[idx] = self.palette.translate_palette(gpu_buffer[idx]);
            }
        }

        self.window
            .update_with_buffer(&self.display, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Something went wrong");

        use crate::console::gui::input;
        let p1 = bus.p1_as_ref();
        input::update_input(&mut self.window, p1);
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open()
    }
}
