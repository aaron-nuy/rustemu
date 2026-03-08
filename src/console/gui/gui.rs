use crate::console::constants::SCREEN_HEIGHT;
use crate::console::constants::SCREEN_WIDTH;
use crate::console::gui::gpu::PixelLevel;
use crate::console::gui::input::compute_input_states;

#[cfg(not(efi))]
use minifb::{Window, WindowOptions};

#[cfg(efi)]
use uefi::proto::console::gop::GraphicsOutput;

#[cfg(efi)]
pub struct Window {
    fb_base: *mut u8,
    fb_stride: usize,
    width: usize,
    height: usize,
}

#[cfg(efi)]
impl Window {
    pub fn new() -> Self {
        use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams};
        use uefi::proto::console::gop::GraphicsOutput;

        let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();

        let mut gop = unsafe {
            boot::open_protocol::<GraphicsOutput>(
                OpenProtocolParams {
                    handle: gop_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .unwrap()
        };

        let (width, height) = gop.current_mode_info().resolution();
        let stride = gop.current_mode_info().stride() * 4;
        let fb_base = gop.frame_buffer().as_mut_ptr();

        Self {
            fb_base,
            fb_stride: stride,
            width,
            height,
        }
    }

    pub fn update_with_buffer(
        &mut self,
        buffer: &[u32],
        i_width: usize,
        i_height: usize,
    ) -> Result<(), ()> {
        //let scale = (self.width / i_width).min(self.height / i_height);
        // no scale
        let scale = 1;

        let scaled_width = i_width * scale;
        let scaled_height = i_height * scale;

        let offset_x = (self.width - scaled_width) / 2;
        let offset_y = (self.height - scaled_height) / 2;

        let fb = self.fb_base as *mut u32;
        let stride = self.fb_stride / 4;

        for src_y in 0..i_height {
            for src_x in 0..i_width {
                let pixel_value = buffer[src_y * i_width + src_x];

                let start_x = offset_x + src_x * scale;
                let start_y = offset_y + src_y * scale;

                for dy in 0..scale {
                    for dx in 0..scale {
                        unsafe {
                            *fb.add((start_y + dy) * stride + (start_x + dx)) = pixel_value;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_open(&self) -> bool {
        true
    }
}

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
        match pixel_level {
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
    display: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Gui {
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(efi)] {

                log::info!("Initializing GUI");
                let m_window = Window::new();
            } else {
                let window_options = WindowOptions {
                    resize: false,
                    title: true,
                    scale: minifb::Scale::X4,
                    ..WindowOptions::default()
                };
                let m_window = Window::new("rustemu", SCREEN_WIDTH, SCREEN_HEIGHT, window_options)
                    .expect("Unable to open window");
            }
        }
        Self {
            palette: Palette::default(),
            window: m_window,
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn new_with_pal(pal: Palette) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(efi)] {
                let m_window = Window::new();
            } else {
                let window_options = WindowOptions {
                    resize: false,
                    title: true,
                    scale: minifb::Scale::X4,
                    ..WindowOptions::default()
                };
                let m_window = Window::new("rustemu", SCREEN_WIDTH, SCREEN_HEIGHT, window_options)
                    .expect("Unable to open window");
            }
        }

        Self {
            palette: pal,
            window: m_window,
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn update(&mut self, bus: &mut crate::console::bus::Bus) {
        let gpu_buffer = bus.get_gpu_buffer();

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx = SCREEN_WIDTH * y + x;
                self.display[idx] = self.palette.translate_palette(gpu_buffer[idx]);
            }
        }

        self.window
            .update_with_buffer(&self.display, SCREEN_WIDTH, SCREEN_HEIGHT)
            .expect("Something went wrong");

        #[cfg(not(efi))]
        let (dpad, buttons) = compute_input_states(&self.window);
        #[cfg(efi)]
        let (dpad, buttons) = compute_input_states();
        bus.update_input_state(dpad, buttons);
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open()
    }
}
