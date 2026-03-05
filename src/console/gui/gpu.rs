use crate::console::constants::*;
use crate::console::hw_register::HwRegister::{OBP0, OBP1};
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
    pub fn decode_line_obj(
        in_buff: &[PixelLevel],
        flip_x: bool,
        palette_in: bool,
        prio_in: bool,
        out: &mut [PixelLevel],
        palette_out: &mut [bool],
        prio_out: &mut [bool],
    ) {
        for x in 0..out.len() {
            let x_idx: usize = if flip_x {
                TILE_DIMS as usize - x - 1
            } else {
                x
            };

            if in_buff[x] != PixelLevel::Zero {
                out[x_idx] = in_buff[x];
                palette_out[x_idx] = palette_in;
                prio_out[x_idx] = prio_in;
            }
        }
    }

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
        unsafe { core::mem::transmute(value) }
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
        let data: [u16; 8] = core::array::from_fn(|i| {
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

#[repr(C)]
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
        Self {
            y: bytes[0],
            x: bytes[1],
            tile_index: bytes[2],
            flags: bytes[3],
        }
    }

    pub unsafe fn from_ptr(addr_in_oam: u16, oam_ram: &[u8; OAM_SIZE as usize]) -> &Self {
        unsafe { &*(core::ptr::addr_of!(oam_ram[addr_in_oam as usize]) as *const Self) }
    }

    pub fn screen_x(&self) -> i16 {
        self.x as i16 - 8
    }

    pub fn screen_y(&self) -> i16 {
        self.y as i16 - 16
    }

    pub fn contributes_to_limit(&self, scanline_y: u8, long_sprite: bool) -> bool {
        let height: i16 = if long_sprite { 16 } else { 8 };
        let sy = self.screen_y();
        sy <= (scanline_y as i16) && (scanline_y as i16) < (sy + height)
    }

    pub fn should_draw(&self, scanline_y: u8, long_sprite: bool) -> bool {
        self.contributes_to_limit(scanline_y, long_sprite) && self.x != 0 && self.x < 168
    }

    pub fn get_tile_y_to_display(&self, scanline_y: u8, long_sprite: bool) -> Option<u8> {
        if (!self.should_draw(scanline_y, long_sprite)) {
            return None;
        }

        let flipped_y = self.flags & OAMFlagMask::YFlip as u8 != 0;

        let height = if long_sprite {
            TILE_DIMS * 2
        } else {
            TILE_DIMS
        };
        let local_y = scanline_y as i16 - self.screen_y();

        if !flipped_y {
            Some(local_y as u8)
        } else {
            Some(height as u8 - 1 - local_y as u8)
        }
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
    pub buffer: [PixelLevel; SCREEN_WIDTH * SCREEN_HEIGHT],
    bg_line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    wd_line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    // Need to store priority and palette flags to use during rendering
    oam_line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    oam_prio: [bool; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    oam_palette: [bool; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
    start_vblank: bool
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            dots: 0,
            gpu_mode: GpuMode::HBlank,
            vram: [0; VRAM_SIZE as usize],
            buffer: [PixelLevel::Zero; SCREEN_WIDTH * SCREEN_HEIGHT],
            bg_line: [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            wd_line: [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            oam_line: [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            oam_prio: [false; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            oam_palette: [false; TILE_MAP_DIMS as usize * TILE_DIMS as usize],
            start_vblank: false,
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
        &mut self,
        curr_ly: u8,
        scy: u8,
        use_tile_map_2: bool,
        no_signed_addressing: bool,
    ) {
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

        self.bg_line = line;
    }

    fn extract_wd_line(
        &mut self,
        curr_ly: u8,
        wy: u8,
        use_tile_map_2: bool,
        no_signed_addressing: bool,
    ) {
        let tile_map_offset: usize = if use_tile_map_2 {
            (TILE_MAP_2_BEGIN - VRAM_BEGIN) as usize
        } else {
            (TILE_MAP_1_BEGIN - VRAM_BEGIN) as usize
        };

        let mut line: [PixelLevel; TILE_MAP_DIMS as usize * TILE_DIMS as usize] =
            [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize];

        let adjusted_y = (curr_ly as i16) - (wy as i16);

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

        self.wd_line = line;
    }

    fn translate_pixel_level_bgp(&self, pixel_level: PixelLevel, bgp: u8) -> PixelLevel {
        PixelLevel::from((bgp >> (pixel_level as u8 * 2)) & 0b11)
    }

    fn translate_pixel_level_other(
        &self,
        pixel_level: PixelLevel,
        obp_1: bool,
        hw_registers: &HwRegisters,
    ) -> PixelLevel {
        let register = if obp_1 {
            hw_registers.read_from_register(OBP1)
        } else {
            hw_registers.read_from_register(OBP0)
        };

        PixelLevel::from((register >> (pixel_level as u8 * 2)) & 0b11)
    }

    fn decide_palette(
        bg_i: PixelLevel,
        bg_p: PixelLevel,
        wd_i: Option<PixelLevel>,
        wd_p: Option<PixelLevel>,
        obj_i: PixelLevel,
        obj_p: PixelLevel,
        bg_enabled: bool,
        wd_enabled: bool,
        obj_enabled: bool,
        obj_priority: bool,
    ) -> PixelLevel {
        let draw_obj = obj_enabled && obj_i != PixelLevel::Zero;
        let draw_window = wd_enabled && wd_i.is_some() && wd_p.is_some();

        let bg_wind_pixel = if bg_enabled {
            if draw_window { wd_p.unwrap() } else { bg_p }
        } else {
            PixelLevel::Zero
        };
        let bg_wind_index = if bg_enabled {
            if draw_window { wd_i.unwrap() } else { bg_i }
        } else {
            PixelLevel::Zero
        };

        let bg_wind_pixel_over_obj = obj_priority && (bg_wind_index != PixelLevel::Zero);

        if bg_wind_pixel_over_obj || !draw_obj {
            bg_wind_pixel
        } else {
            obj_p
        }
    }

    fn extract_obj_line(
        &mut self,
        long_sprite: bool,
        curr_ly: u8,
        oam_ram: &[u8; OAM_SIZE as usize],
    ) {
        self.oam_line = [PixelLevel::Zero; TILE_MAP_DIMS as usize * TILE_DIMS as usize];
        self.oam_prio = [false; TILE_MAP_DIMS as usize * TILE_DIMS as usize];
        self.oam_palette = [false; TILE_MAP_DIMS as usize * TILE_DIMS as usize];

        // TODO: implement object on object priority
        // From pandocs:
        // the smaller the X coordinate, the higher the priority. When X coordinates are identical,
        // the object located first in OAM has higher priority
        let mut obj_count = 0;
        for oam_index in 0..OAM_SIZE / OAM_ENTRY_SIZE {
            if obj_count >= MAX_OJBS_PER_SCANLINE {
                break;
            }

            let oam_entry_addr = oam_index * OAM_ENTRY_SIZE;

            let oam_entry = unsafe { OAMEntry::from_ptr(oam_entry_addr, oam_ram) };

            if !oam_entry.contributes_to_limit(curr_ly, long_sprite) {
                continue;
            }

            obj_count += 1;

            let obj_local_y = oam_entry.get_tile_y_to_display(curr_ly, long_sprite);
            if obj_local_y.is_none() {
                continue;
            }

            let x_flipped = (oam_entry.flags & OAMFlagMask::XFlip as u8) != 0;
            let priority = (oam_entry.flags & OAMFlagMask::Priority as u8) != 0;
            let palette = (oam_entry.flags & OAMFlagMask::DmgPalette as u8) != 0;

            let first_half_idx = if !long_sprite {
                oam_entry.tile_index
            } else {
                oam_entry.tile_index & !0b1
            };
            let second_half_idx = first_half_idx.wrapping_add(1);

            let use_first_half = obj_local_y.unwrap() < TILE_DIMS as u8;

            let tile_addr = if use_first_half {
                self.get_tile_addr_adjusted(first_half_idx, true, true)
            } else {
                self.get_tile_addr_adjusted(second_half_idx, true, true)
            };

            let screen_x = oam_entry.screen_x();
            let start_x = screen_x.max(0);
            let end_x = (screen_x + TILE_DIMS as i16 - 1).min(SCREEN_WIDTH as i16 - 1);

            let local_start_x = if screen_x < 0 {
                screen_x.abs()
            } else {
                0
            };
            let local_end_x = if end_x as usize >= SCREEN_WIDTH {
                (SCREEN_WIDTH as i16 - 1 - end_x + TILE_DIMS as i16) as u8
            } else {
                TILE_DIMS as u8 - 1
            };

            let mut decode_output = [PixelLevel::Zero; TILE_DIMS as usize];
            let tile_local_y = obj_local_y.unwrap() % TILE_DIMS as u8;
            let base = tile_addr as usize + TILE_LINE_BYTE_SIZE * tile_local_y as usize;
            PixelLevel::decode_line(
                self.vram[base..base + 2].try_into().unwrap(),
                &mut decode_output,
            );

            let extracted_line_slice =
                &mut decode_output[local_start_x as usize..=local_end_x as usize];
            let oam_line_slice = &mut self.oam_line[start_x as usize..=end_x as usize];
            let oam_prio_slice = &mut self.oam_prio[start_x as usize..=end_x as usize];
            let oam_palette_slice = &mut self.oam_palette[start_x as usize..=end_x as usize];

            PixelLevel::decode_line_obj(
                extracted_line_slice,
                x_flipped,
                palette,
                priority,
                oam_line_slice,
                oam_palette_slice,
                oam_prio_slice,
            );
        }
    }

    fn handle_mode_3(&mut self, curr_y: u8, curr_x: u8, hw_registers: &mut HwRegisters, lcdc: u8) {
        let bg_enabled = lcdc & (LCDCFlag::BackgroundEnabled as u8) != 0;
        let wd_enabled = lcdc & (LCDCFlag::WindowEnabled as u8) != 0;
        let obj_enabled = lcdc & (LCDCFlag::ObjEnabled as u8) != 0;

        let scx = hw_registers.read_from_register(HwRegister::SCX);
        let wx = hw_registers.read_from_register(HwRegister::WX);

        let bg_x = curr_x.wrapping_add(scx);
        let wd_x = (curr_x as i16) - (wx as i16) + 7;

        let obj_i = self.oam_line[curr_x as usize];
        let obj_prio = self.oam_prio[curr_x as usize];
        let obj_palette = self.oam_palette[curr_x as usize];
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
        let obj_p = self.translate_pixel_level_other(obj_i, obj_palette, hw_registers);

        let out_c = Self::decide_palette(
            bg_i,
            bg_p,
            opt_wd_i,
            opt_wd_p,
            obj_i,
            obj_p,
            bg_enabled,
            wd_enabled,
            obj_enabled,
            obj_prio,
        );

        self.buffer[curr_y as usize * SCREEN_WIDTH + curr_x as usize] = out_c;
    }

    pub fn tick(&mut self, hw_registers: &mut HwRegisters, oam_ram: &[u8; OAM_SIZE as usize]) {
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
                self.start_vblank = true;
                hw_registers.request_interrupt(Interrupt::VBlank);
            } else {
                self.start_vblank = false;
            }
        } else {
            if (scanline_dots < OAM_SCAN_DOT_LENGTH) {
                // OAMScan (Mode 2)
                self.gpu_mode = GpuMode::OamScan;

                if (scanline_dots == 0) {
                    let long_sprite = lcdc & LCDCFlag::LongSpriteEnabled as u8 != 0;
                    self.extract_obj_line(long_sprite, ly, oam_ram);
                }
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
                    self.extract_bg_line(ly, scy, tm2_bg, no_signed_addressing);
                    self.extract_wd_line(ly, wy, tm2_wd, no_signed_addressing);
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

    pub fn is_vblank_started(&self) -> bool {
        self.start_vblank
    }
}
