use crate::console::bus::Bus;
use crate::console::constants::*;
use crate::console::hw_register::{HwRegister, HwRegisters};
use crate::console::interrupt::Interrupt;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PixelLevel {
    Zero = 0b00,
    One = 0b01,
    Two = 0b10,
    Three = 0b11,
}

impl PixelLevel {
    #[inline(always)]
    pub fn decode_line(encoded_line: &[u8; 2], out: &mut [PixelLevel; TILE_DIMS as usize]) {
        for x in 0..TILE_DIMS as usize {
            let shift = 7 - x;
            let msb = (encoded_line[0] >> shift) & 0b1;
            let lsb = (encoded_line[1] >> shift) & 0b1;
            out[x] = PixelLevel::from((msb << 1) | lsb);
        }
    }
}

impl From<u8> for PixelLevel {
    fn from(value: u8) -> Self {
        debug_assert!(value <= 0b11);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Tile {
    pub data: [u16; 8],
}

impl Tile {
    pub fn new() -> Self {
        Self { data: [0; 8] }
    }

    pub fn from_bytes_8(bytes: [u8; 16]) -> Self {
        let data: [u16; 8] = std::array::from_fn(|i| {
            let lo = bytes[i * 2];
            let hi = bytes[i * 2 + 1];
            u16::from_le_bytes([lo, hi])
        });

        Self { data }
    }
}

#[repr(u8)]
pub enum LCDCFlag {
    BackgroundEnabled = 0b0000_0001,
    ObjEnabled = 0b0000_0010,
    LongSpriteEnabled = 0b0000_0100,
    UseTileMap2Bg = 0b0000_1000,
    NoSignedAddressing = 0b0001_0000,
    WindowEnabled = 0b0010_0000,
    UseTimeMap2Wd = 0b0100_0000,
    GpuEnabled = 0b1000_0000,
}

#[repr(u8)]
pub enum STATFlag {
    PPUMode = 0b0000_0011,
    LYEqLYC = 0b0000_0100,
    Mode0IntSelect = 0b0000_1000,
    Mode1IntSelect = 0b0001_0000,
    Mode2IntSelect = 0b0010_0000,
    LYCIntSelect = 0b0100_0000,
    __UNUSED__ = 0b1000_0000,
}

#[repr(u8)]
enum OAMFlagMask {
    CgbPalette = 0b0000_0111, // unused dmg
    Bank = 0b0000_1000,       // unused dmg
    DmgPalette = 0b0001_0000,
    XFlip = 0b0010_0000,
    YFlip = 0b0100_0000,
    Priority = 0b1000_0000, // If 1 bg/window are drawn drawn on top of it, only indices 1,2,3 are drawn on top
}
struct OAMEntry {
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub flags: u8,
}

impl OAMEntry {
    pub fn new() -> Self {
        Self {
            y: 0,
            x: 0,
            tile_index: 0,
            flags: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        let y = bytes[0];
        let x = bytes[1];
        let tile_index = bytes[2];
        let flags = bytes[3];
        Self {
            y,
            x,
            tile_index,
            flags,
        }
    }

    pub fn screen_x(&self) -> i16 {
        (self.x as i8 as i16) - 8
    }

    pub fn screen_y(&self) -> i16 {
        (self.y as i8 as i16) - 16
    }

    pub fn contributes_to_limit(&self, long_sprite: bool) -> bool {
        let min_y_to_show = if long_sprite { 0 } else { 8 };

        self.y < 160 && self.y > min_y_to_show
    }

    pub fn should_draw(&self, long_sprite: bool) -> bool {
        self.contributes_to_limit(long_sprite) && self.x != 0 && self.x < 168
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GpuMode {
    HBlank = 0b00,
    VBlank = 0b01,
    OamScan = 0b10,
    Drawing = 0b11,
}

pub struct Gpu {
    dots: u64,
    pub gpu_mode: GpuMode,
    pub vram: [u8; VRAM_SIZE as usize],
    pub oam: [u8; OAM_SIZE as usize],
    pub buffer: [PixelLevel; SCREEN_WIDTH * SCREEN_HEIGHT],
    bg_line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    wd_line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            dots: 0,
            gpu_mode: GpuMode::HBlank,
            vram: [0; VRAM_SIZE as usize],
            oam: [0; OAM_SIZE as usize],
            buffer: [PixelLevel::Zero; SCREEN_WIDTH * SCREEN_HEIGHT],
            bg_line: [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            wd_line: [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
        }
    }

    pub fn write_to_vram(&mut self, addr: u16, value: u8) {
        self.vram[addr as usize] = value;
    }

    pub fn read_from_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }

    fn get_tile_addr_adjusted(
        &self,
        index: u8,
        objects: bool,
        no_use_signed_addressing: bool,
    ) -> u16 {
        let signed = !objects && !no_use_signed_addressing;

        if !signed {
            const OFFSET: u16 = TILE_BLOCK_0 - VRAM_BEGIN;
            OFFSET + (index as u16) * TILE_SIZE
        } else {
            const SIGNED_IDX_OFFSET: u8 = 128;
            const OFFSET: u16 = TILE_BLOCK_1 - VRAM_BEGIN;
            OFFSET + (index.wrapping_add(SIGNED_IDX_OFFSET) as u16) * TILE_SIZE
        }
    }

    fn extract_bg_line(
        &self,
        curr_ly: u8,
        scy: u8,
        use_tile_map_2: bool,
        no_signed_addressing: bool,
    ) -> [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize] {
        let tile_map_offset: usize = if use_tile_map_2 {
            (TILE_MAP_2_BEGIN - VRAM_BEGIN) as usize
        } else {
            (TILE_MAP_1_BEGIN - VRAM_BEGIN) as usize
        };

        let mut line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize] =
            [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize];

        let adjusted_y = curr_ly.wrapping_add(scy);
        let tile_y = adjusted_y % TILE_DIMS as u8;
        let tile_idx_y = adjusted_y as usize / TILE_DIMS as usize;
        let line_offset = tile_map_offset + TILE_MAP_DIMS as usize * tile_idx_y;

        for tile_idx_x in 0..TILE_MAP_DIMS as usize {
            let tile_idx = self.vram[line_offset + tile_idx_x];

            let tile_addr = self.get_tile_addr_adjusted(tile_idx, false, no_signed_addressing);
            let base = tile_addr as usize + TILE_LINE_BYTE_SIZE * tile_y as usize;
            let start = tile_idx_x * TILE_DIMS as usize;

            let line_bytes: &[u8; 2] = self.vram[base..base + 2].try_into().unwrap();
            let out: &mut [PixelLevel; TILE_DIMS as usize] = (&mut line
                [start..start + TILE_DIMS as usize])
                .try_into()
                .unwrap();

            PixelLevel::decode_line(line_bytes, out);
        }

        line
    }

    fn extract_wd_line(
        &self,
        curr_ly: u8,
        wy: u8,
        use_tile_map_2: bool,
        no_signed_addressing: bool,
    ) -> [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize] {
        let tile_map_offset: usize = if use_tile_map_2 {
            (TILE_MAP_2_BEGIN - VRAM_BEGIN) as usize
        } else {
            (TILE_MAP_1_BEGIN - VRAM_BEGIN) as usize
        };

        let mut line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize] =
            [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize];

        let adjusted_y = (curr_ly as i16) - (wy as i16);

        if adjusted_y < 0 {
            return line;
        }

        let adjusted_y: u8 = adjusted_y as u8;

        let tile_y = adjusted_y % TILE_DIMS as u8;
        let tile_idx_y = adjusted_y as usize / TILE_DIMS as usize;
        let line_offset = tile_map_offset + TILE_MAP_DIMS as usize * tile_idx_y;

        for tile_idx_x in 0..TILE_MAP_DIMS as usize {
            let tile_idx = self.vram[line_offset + tile_idx_x];

            let tile_addr = self.get_tile_addr_adjusted(tile_idx, false, no_signed_addressing);
            let base = tile_addr as usize + TILE_LINE_BYTE_SIZE * tile_y as usize;
            let start = tile_idx_x * TILE_DIMS as usize;

            let line_bytes: &[u8; 2] = self.vram[base..base + 2].try_into().unwrap();
            let out: &mut [PixelLevel; TILE_DIMS as usize] = (&mut line
                [start..start + TILE_DIMS as usize])
                .try_into()
                .unwrap();

            PixelLevel::decode_line(line_bytes, out);
        }

        line
    }

    fn translate_pixel_level_bgp(&self, pixel_level: PixelLevel, bgp: u8) -> PixelLevel {
        PixelLevel::from((bgp >> (pixel_level as u8 * 2)) & 0b11)
    }

    fn translate_pixel_level_other(
        &self,
        pixel_level: PixelLevel,
        obp_1: bool,
        bus: &Bus,
    ) -> Option<PixelLevel> {
        // Pixel index zero is transparent
        if pixel_level == PixelLevel::Zero {
            return None;
        }

        let register_addr = if obp_1 {
            HwRegister::OBP1 as u16
        } else {
            HwRegister::OBP0 as u16
        };
        let register = bus.read_from_8b(register_addr);
        Some(PixelLevel::from(
            (register >> (pixel_level as u8 * 2)) & 0b11,
        ))
    }

    fn handle_mode_3(&mut self, curr_y: u8, curr_x: u8, hw_registers: &mut HwRegisters, lcdc: u8) {
        let bg_enabled = lcdc & (LCDCFlag::BackgroundEnabled as u8) != 0;
        let wd_enabled = lcdc & (LCDCFlag::WindowEnabled as u8) != 0;
        let obj_enabled = lcdc & (LCDCFlag::ObjEnabled as u8) != 0;

        let scx = hw_registers.read_from_register(HwRegister::SCX);
        let wx = hw_registers.read_from_register(HwRegister::WX);

        let bg_x = curr_x.wrapping_add(scx);
        let wd_x = (curr_x as i16) - (wx as i16) + 7;

        let bg_i = self.bg_line[bg_x as usize];
        let opt_wd_i = if wd_x >= 0 && wd_x < SCREEN_WIDTH as i16 {
            Some(self.wd_line[wd_x as usize])
        } else {
            None
        };

        let bg_p =
            self.translate_pixel_level_bgp(bg_i, hw_registers.read_from_register(HwRegister::BGP));
        let opt_wd_p = opt_wd_i.map(|i| {
            self.translate_pixel_level_bgp(i, hw_registers.read_from_register(HwRegister::BGP))
        });

        let mut out_c = PixelLevel::Zero;
        if bg_enabled {
            out_c = if wd_enabled {
                opt_wd_p.unwrap_or(bg_p)
            } else {
                bg_p
            };
        }

        self.buffer[curr_y as usize * SCREEN_WIDTH + curr_x as usize] = out_c;
    }

    pub fn tick(&mut self, hw_registers: &mut HwRegisters) {
        let lcdc = hw_registers.read_from_register(HwRegister::LCDC);

        if lcdc & (LCDCFlag::GpuEnabled as u8) == 0 {
            self.dots = 0;
            self.gpu_mode = GpuMode::HBlank;
            // TODO: Should update ly to zero?
            // TODO: Screen should be blanked out when gpu is off
            return;
        }

        let mut ly = hw_registers.read_from_register(HwRegister::LY);

        ly = ((self.dots / DOTS_PER_SCANLINE) % NUMBER_SCANLINES) as u8;
        let scanline_dots = self.dots % DOTS_PER_SCANLINE;

        // update ly
        hw_registers.write_to_register(HwRegister::LY, ly);

        if (ly >= SCREEN_HEIGHT as u8) {
            // VBlank (Mode 1)
            self.gpu_mode = GpuMode::VBlank;
            hw_registers.handle_stat_line_mode1_cond();
            if (ly == SCREEN_HEIGHT as u8 && scanline_dots == 0) {
                hw_registers.request_interrupt(Interrupt::VBlank);
            }
        } else {
            if (scanline_dots < OAM_SCAN_DOT_LENGTH) {
                // OAMScan (Mode 2)
                self.gpu_mode = GpuMode::OamScan;
                hw_registers.handle_stat_line_mode2_cond();
            } else if (scanline_dots < OAM_SCAN_DOT_LENGTH + (SCREEN_WIDTH + 12) as u64) {
                // 12 is min additional time spent fetching tiles and so on
                // Drawing (Mode 3)
                self.gpu_mode = GpuMode::Drawing;

                if scanline_dots == OAM_SCAN_DOT_LENGTH {
                    let scy = hw_registers.read_from_register(HwRegister::SCY);
                    let wy = hw_registers.read_from_register(HwRegister::WY);
                    let no_signed_addressing = lcdc & (LCDCFlag::NoSignedAddressing as u8) != 0;
                    let tm2_bg = lcdc & (LCDCFlag::UseTileMap2Bg as u8) != 0;
                    let tm2_wd = lcdc & (LCDCFlag::UseTimeMap2Wd as u8) != 0;
                    self.bg_line = self.extract_bg_line(ly, scy, tm2_bg, no_signed_addressing);
                    self.wd_line = self.extract_wd_line(ly, wy, tm2_wd, no_signed_addressing);
                }

                let mode_3_dot = (scanline_dots - OAM_SCAN_DOT_LENGTH) as usize;

                if mode_3_dot < SCREEN_WIDTH {
                    self.handle_mode_3(ly, mode_3_dot as u8, hw_registers, lcdc);
                } else {
                    // Do nothing, simulate extra time
                }
            } else {
                // Hblank (Mode 0)
                self.gpu_mode = GpuMode::HBlank;
                hw_registers.handle_stat_line_mode0_cond();
            }
        }

        self.dots += 1;
    }
}
