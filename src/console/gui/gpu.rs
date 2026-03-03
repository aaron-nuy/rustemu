use crate::console::bus::Bus;
use crate::console::constants::*;
use crate::console::hw_register::{HwRegister, HwRegisters};
use std::ops::{BitAnd, Sub};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PixelLevel {
    Zero = 0b00,
    One = 0b01,
    Two = 0b10,
    Three = 0b11,
}

impl PixelLevel {
    pub fn from_byte(shade: u8) -> PixelLevel {
        assert!(shade <= 0b11);
        match (shade) {
            0 => PixelLevel::Zero,
            1 => PixelLevel::One,
            2 => PixelLevel::Two,
            3 => PixelLevel::Three,
            _ => unreachable!(),
        }
    }

    pub fn to_int(&self) -> u8 {
        match (self) {
            PixelLevel::Zero => 255,
            PixelLevel::One => 170,
            PixelLevel::Two => 85,
            PixelLevel::Three => 0,
        }
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

    pub fn read_pixel(&self, x: usize, y: usize) -> PixelLevel {
        assert!(x < 8);
        assert!(y < 8);

        let line = self.data[y];

        let first_half: u8 = (line & 0xFF) as u8;
        let second_half: u8 = ((line >> 8) & 0xFF) as u8;

        // the most significant bit represents the leftmost pixel so we reverse bit order
        let shift = 7 - x;

        let msb = (first_half >> shift) & 0b1;
        let lsb = (second_half >> shift) & 0b1;

        PixelLevel::from_byte((msb << 1) | lsb)
    }
}

#[repr(u8)]
enum LCDCFlag {
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
enum OAMFlagMask {
    CgbPalette = 0b0000_0111,
    Bank = 0b0000_1000,
    DmgPalette = 0b0001_0000,
    XFlip = 0b0010_0000,
    YFlip = 0b0100_0000,
    Priority = 0b1000_0000,
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

#[repr(u8)]
enum GpuMode {
    HBlank = 0b00,
    VBlank = 0b01,
    OamScan = 0b10,
    Drawing = 0b11,
}

pub struct Gpu {
    dots: u64,
    pub vram: [u8; VRAM_SIZE as usize],
    pub oam: [u8; OAM_SIZE as usize],
    pub buffer: [PixelLevel; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Gpu {
    pub fn new() -> Self {
        Self {
            dots: 0,
            vram: [0; VRAM_SIZE as usize],
            oam: [0; OAM_SIZE as usize],
            buffer: [PixelLevel::Zero; SCREEN_WIDTH * SCREEN_HEIGHT],
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

    fn extract_tile_map(
        &self,
        use_tile_map_2: bool,
        objects: bool,
        no_use_signed_addressing: bool,
    ) -> [Tile; TILE_MAP_SIZE as usize] {
        let tile_map_offset: usize = if use_tile_map_2 {
            (TILE_MAP_2_BEGIN - VRAM_BEGIN) as usize
        } else {
            (TILE_MAP_1_BEGIN - VRAM_BEGIN) as usize
        };

        let mut tile_map = [Tile::default(); TILE_MAP_SIZE as usize];

        for i in 0..TILE_MAP_SIZE as usize {
            let tile_idx = self.vram[tile_map_offset + i];
            let tile_addr =
                self.get_tile_addr_adjusted(tile_idx, objects, no_use_signed_addressing);
            let tile_bytes = &self.vram[tile_addr as usize..(tile_addr + TILE_SIZE) as usize];
            tile_map[i] = Tile::from_bytes_8(tile_bytes.try_into().unwrap());
        }

        tile_map
    }

    fn translate_pixel_level_bgp(
        &self,
        pixel_level: PixelLevel,
        hw_registers: &HwRegisters,
    ) -> PixelLevel {
        let bgp = hw_registers.read_from_register(HwRegister::BGP);
        PixelLevel::from_byte((bgp >> (pixel_level as u8 * 2)) & 0b11)
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
        Some(PixelLevel::from_byte(
            (register >> (pixel_level as u8 * 2)) & 0b11,
        ))
    }

    fn render_background(
        &mut self,
        background_tile_map: &[Tile; TILE_MAP_SIZE as usize],
        hw_registers: &HwRegisters,
    ) {
        let scx = hw_registers.read_from_register(HwRegister::SCX);
        let scy = hw_registers.read_from_register(HwRegister::SCY);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let adjusted_x = (x as u8).wrapping_add(scx);
                let adjusted_y = (y as u8).wrapping_add(scy);

                let tile_idx_x = adjusted_x / (TILE_DIMS as u8);
                let tile_idx_y = adjusted_y / (TILE_DIMS as u8);

                let tile = background_tile_map
                    [tile_idx_y as usize * TILE_MAP_DIMS as usize + tile_idx_x as usize];

                let tile_x = adjusted_x % (TILE_DIMS as u8);
                let tile_y = adjusted_y % (TILE_DIMS as u8);

                let pixel_level = tile.read_pixel(tile_x as usize, tile_y as usize);

                self.buffer[y * SCREEN_WIDTH + x] =
                    self.translate_pixel_level_bgp(pixel_level, &hw_registers);
            }
        }
    }

    fn render_window(
        &mut self,
        window_tile_map: &[Tile; TILE_MAP_SIZE as usize],
        hw_registers: &HwRegisters,
    ) {
        let wx = hw_registers.read_from_register(HwRegister::WX);
        let wy = hw_registers.read_from_register(HwRegister::WY);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let window_y: i16 = (y as i16) - (wy as i16);
                let window_x: i16 = (x as i16) - (wx as i16) + 7;

                if window_y < 0 || window_x < 0 {
                    continue;
                }

                // don't allow reading outside tilemap
                if window_y >= (TILE_DIMS as i16 * TILE_MAP_DIMS as i16)
                    || window_x >= (TILE_DIMS as i16 * TILE_MAP_DIMS as i16)
                {
                    continue;
                }

                let tile_idx_x = window_x / (TILE_DIMS as i16);
                let tile_idx_y = window_y / (TILE_DIMS as i16);

                let tile_x = window_x % (TILE_DIMS as i16);
                let tile_y = window_y % (TILE_DIMS as i16);

                let tile = window_tile_map
                    [tile_idx_y as usize * TILE_MAP_DIMS as usize + tile_idx_x as usize];

                let pixel_level = tile.read_pixel(tile_x as usize, tile_y as usize);

                self.buffer[y * SCREEN_WIDTH + x] =
                    self.translate_pixel_level_bgp(pixel_level, &hw_registers);
            }
        }
    }

    pub fn tick(&mut self, hw_registers: &mut HwRegisters) {
        let lcdc = hw_registers.read_from_register(HwRegister::LCDC);

        if lcdc & (LCDCFlag::GpuEnabled as u8) == 0 {
            self.dots = 0;
            // TODO: Screen should be blanked out when gpu is off
            return;
        }

        let mut ly = hw_registers.read_from_register(HwRegister::LY);
        
        ly = ((self.dots / DOTS_PER_SCANLINE) % NUMBER_SCANLINES) as u8;
        let scanline_dots = self.dots % DOTS_PER_SCANLINE;
        
        // update ly
        hw_registers.write_to_register(HwRegister::LY, ly);

        if (ly >= SCREEN_HEIGHT as u8) {
            // Vblank period
            if (scanline_dots == 0) {
                // Trigger vblank interrupt
            }
        } else {

            if (scanline_dots < 80) {
                // OAMScan (Mode 2)
            } else {
                // Drawing (Mode 3)
                let no_use_signed_addressing = lcdc & (LCDCFlag::NoSignedAddressing as u8) != 0;

                if lcdc & (LCDCFlag::BackgroundEnabled as u8) != 0 {
                    let use_tile_map_2_bg = lcdc & (LCDCFlag::UseTileMap2Bg as u8) != 0;
                    let background_tile_map =
                        self.extract_tile_map(use_tile_map_2_bg, false, no_use_signed_addressing);
                    self.render_background(&background_tile_map, &hw_registers);
                    if lcdc & (LCDCFlag::WindowEnabled as u8) != 0 {
                        let use_tile_map_2_wd = lcdc & (LCDCFlag::UseTimeMap2Wd as u8) != 0;
                        let window_tile_map = self.extract_tile_map(
                            use_tile_map_2_wd,
                            false,
                            no_use_signed_addressing,
                        );
                        self.render_window(&window_tile_map, &hw_registers);
                    }
                }

                if lcdc & (LCDCFlag::ObjEnabled as u8) != 0 {
                    let object_tile_map =
                        self.extract_tile_map(false, true, no_use_signed_addressing);
                }
            }
        }

        self.dots += 1;
    }
}
