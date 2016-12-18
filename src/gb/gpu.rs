use std::vec::Vec;
use std::cmp;

use ::mode;
use ::bits;
use ::frame::Frame;

/// Display width
pub const WIDTH: usize = 160;

/// Display height
pub const HEIGHT: usize = 144;

#[derive(Default)]
pub struct GPU {
    /// Mode (machine)
    mode: Option<mode::Mode>,

    /// Callback: Refresh (v-blank)
    on_refresh: Option<Box<FnMut(Frame) -> ()>>,

    // TODO: This should not be public but it is for my hacked SDL usage
    /// Pixel data (for current "frame") buffer that is rewritten line-by-line during H-Blank
    pub framebuffer: Vec<u8>,

    /// [0x8000 - 0x9FFF] Video RAM (VRAM) — 8 KiB x 2 Banks (1 Bank in GB)
    vram: Vec<u8>,

    /// [0xFF4F] Video RAM Bank (VBK) 0 - 1 (R/W; CGB only)
    vram_bank: u8,

    /// [0xFE00 — 0xFE9F] Sprite Attribute Table (OAM) — 160 Bytes
    pub oam: Vec<u8>,

    /// STAT IRQ; current value
    /// Actual interrupt is triggered when this goes from 0 to 1
    stat_irq: bool,

    /// T-Cycle counter for GPU mode
    cycles: u32,

    /// T-Cycle counter for Mode 3 (variable length)
    m3_cycles: u32,

    /// Priority cache for rendering (1byte per pixel)
    ///     Bit 0 -> 1 = Background/window was rendered at this pixel
    ///     Bit 1 -> 1 = Sprite was rendered at this pixel
    ///     Bit 2 -> 1 = Background/window have priority over sprite
    priority_cache: Vec<u8>,

    /// Sprite X Cache (1byte per pixel that = sprite_x)
    sprite_x_cache: Vec<u8>,

    /// Sprite Stall Buckets
    sprite_stall_buckets: Vec<u8>,

    /// Current line of the window that is being rendered
    /// Essentially if the window gets disabled after LY=10, then re-enabled before LY=40, the
    /// window will render LY=11's window line on LY=40.
    window_line: u8,

    /// [0xFF44] LCDC Y-Coordinate (LY) (R)
    ly: u8,

    /// LY Comparison Timer
    ///    When a change to LY happens; a 4 T-Cycle timer begins. After expiring
    ///    LY is available and the STAT IF is flagged if enabled and
    ///    matched.
    lyc_timer: u8,

    /// [0xFF41] - STAT - LCDC Status (R/W)
    ///     Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    lyc_irq_enable: bool,

    /// [0xFF41] - STAT - LCDC Status (R/W)
    ///     Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    m2_irq_enable: bool,

    /// [0xFF41] - STAT - LCDC Status (R/W)
    ///     Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    m1_irq_enable: bool,

    /// [0xFF41] - STAT - LCDC Status (R/W)
    ///     Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    m0_irq_enable: bool,

    /// [0xFF41] - STAT - LCDC Status (R/W)
    /// Bit 1-0 - Mode Flag
    ///   0: H-Blank
    ///   1: V-Blank
    ///   2: Searching OAM-RAM
    ///   3: Transferring Data to LCD Driver.
    lcd_mode: u8,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 7 - LCD Display Enable             (0=Off, 1=On)
    lcd_enable: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    window_tile_map_select: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 5 - Window Display Enable          (0=Off, 1=On)
    window_enable: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    tile_data_select: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    background_tile_map_select: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    sprite_16: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    sprite_enable: bool,

    /// [0xFF40] - LCDC - LCD Control
    ///   Bit 0 - BG Display
    ///    For  GB -> 0=Off, 1=On
    ///    For CGB -> 0=Background/Window have no priority, 1=Normal priority
    background_display: bool,

    /// [0xFF42] SCY - Scroll Y (R/W)
    scy: u8,

    /// [0xFF43] SCX - Scroll X (R/W)
    scx: u8,

    /// [0xFF45] LYC - LY Compare (R/W)
    lyc: u8,

    /// [0xFF4A] WY - Window Y Position (R/W)
    wy: u8,

    /// [0xFF4B] WX - Window X Position (- 7) (R/W)
    wx: u8,

    /// [0xFF47] BGP — BG Palette Data (R/W)
    bgp: u8,

    /// [0xFF48] OBP0 - Object Palette 0 Data (R/W)
    obp0: u8,

    /// [0xFF49] OBP1 - Object Palette 1 Data (R/W)
    obp1: u8,

    /// [0xFF68] BCPS/BGPI - CGB Mode Only - Background Palette Index
    ///    Bit 0-5   Index (00-3F)
    bcpi: u8,

    /// [0xFF68] BCPS/BGPI - CGB Mode Only - Background Palette Index
    ///    Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
    bcpi_ai: bool,

    /// [0xFF69] BCPD/BGPD - CGB Mode Only - Background Palette Data (x8)
    ///    Bit 0-4   Red Intensity   (00-1F)
    ///    Bit 5-9   Green Intensity (00-1F)
    ///    Bit 10-14 Blue Intensity  (00-1F)
    bcpd: Vec<u8>,

    /// [0xFF6A] OCPS/OBPI - CGB Mode Only - Sprite Palette Index
    ///    Bit 0-5   Index (00-3F)
    ocpi: u8,

    /// [0xFF6A] - OCPS/OBPI - CGB Mode Only - Sprite Palette Index
    ///    Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
    ocpi_ai: bool,

    /// [0xFF6B] - OCPD/OBPD - CGB Mode Only - Sprite Palette Data
    ///    Same as BCDP but for the sprite palette
    ocpd: Vec<u8>,
}

impl GPU {
    pub fn set_on_refresh(&mut self, callback: Box<FnMut(Frame) -> ()>) {
        self.on_refresh = Some(callback);
    }

    /// Step
    pub fn step(&mut self, if_: &mut u8) {
        // TODO: What do we do when the LCD is disabled?
        if !self.lcd_enable {
            return;
        }

        // Mode 2 and 0 IRQ are raised some cycles before the GPU actually enters the
        // respective modes; we use this variable to mark the mode being compared
        // when we check for a IRQ. If it is still 0xFF it gets set to `self.lcd_mode`.
        let mut mode_cmp = 0xFF;

        // LY is compared to LYC and the "coincidence" IRQ is raised. However, when the
        // LY register is updated, there is a 4 T-Cycle delay until it can be compared.
        if self.lyc_timer > 0 {
            self.lyc_timer -= 1;
        }

        // Increment cycle counter (reset to 0 at the beginning of each scanline)
        self.cycles += 1;

        if self.lcd_mode == 0 && self.cycles == 5 {
            // Lines 0-144 start each scanline in mode 0 for 4 cycles
            if self.ly < 144 {
                // Proceed to mode 2 — Searching OAM-RAM
                self.lcd_mode = 2;
            } else {
                // Proceed to mode 1 — V-Blank
                self.lcd_mode = 1;
                self.window_line = 0;

                // Trigger VBL interrupt
                (*if_) |= 0x1;

                // Trigger the front-end to refresh the scren
                if let &mut Some(ref mut on_refresh) = &mut self.on_refresh {
                    (on_refresh)(Frame {
                        data: &self.framebuffer,
                        width: WIDTH,
                        height: HEIGHT,
                        pitch: WIDTH * 4,
                    });
                }
            }
        } else if self.lcd_mode == 0 && self.cycles >= 1 && self.cycles < 5 && self.ly >= 1 &&
                  self.ly <= 143 {
            // Lines 1-143 signal the M2 IRQ 4 T-cycles early
            mode_cmp = 2;
        } else if self.lcd_mode == 2 && self.cycles == 85 {
            // Proceed to mode 3 — Transferring Data to LCD Driver
            self.lcd_mode = 3;

            // Render scanline (at the -start- of mode-3)
            self.render();
        } else if self.lcd_mode == 3 && self.cycles >= ((85 + self.m3_cycles) - 7) &&
                  self.cycles < (85 + self.m3_cycles) {
            // The mode M0 IRQ is signalled 7 T-cycles early
            // TODO: Try and verify that
            mode_cmp = 0;
        } else if self.lcd_mode == 3 && self.cycles == (85 + self.m3_cycles) {
            // Proceed to mode 0 — H-Blank
            self.lcd_mode = 0;
        } else if self.lcd_mode == 0 && self.cycles == 457 {
            // A scanline takes 456 T-Cycles to complete
            // This acts as the 0th scanline for lines 1-143
            self.ly += 1;
            self.lyc_timer = 4;
            self.lcd_mode = 0;
            self.cycles = 1;
        } else if self.lcd_mode == 1 {
            if self.cycles == 457 {
                self.cycles = 1;

                if self.ly == 0 {
                    // Restart process (back to top of LCD)
                    self.lcd_mode = 0;
                } else {
                    self.ly += 1;
                    self.lyc_timer = 4;
                }
            } else if self.ly == 153 && self.cycles == 5 {
                // Scanline 153 spends only 4 T-Cycles with LY == 153
                self.ly = 0;
                self.lyc_timer = 4;
            }
        }

        // The STAT interrupt is fired when the signal TRANSITIONS from 0 TO 1
        // If it _stays_ 1 during a screen mode change then no interrupt is fired.
        if mode_cmp == 0xFF {
            mode_cmp = self.lcd_mode;
        }

        let irq = ((self.lyc_timer == 0) && (self.ly == self.lyc) && self.lyc_irq_enable) ||
                  (mode_cmp == 0 && self.m0_irq_enable) ||
                  (mode_cmp == 2 && self.m2_irq_enable) ||
                  (mode_cmp == 1 && (self.m1_irq_enable || self.m2_irq_enable));

        if !self.stat_irq && irq {
            // Raise interrupt
            (*if_) |= 0x2;
        }

        self.stat_irq = irq;
    }

    /// Reset
    pub fn reset(&mut self, m: mode::Mode) {
        // Assign: Mode
        self.mode = Some(m);

        // Reset: Framebuffer
        self.framebuffer.clear();
        self.framebuffer.resize(WIDTH * HEIGHT * 4, 0xFF);

        // Reset: VRAM
        // TODO: 0x2000 * 1 in GB mode
        self.vram.clear();
        self.vram.resize(0x2000 * 2, 0);

        // Reset: OAM
        self.oam.clear();
        self.oam.resize(160, 0);

        // Reset: BCPD
        // TODO: Do not size in GB mode
        self.bcpd.clear();
        self.bcpd.resize(64, 0);

        // Reset: OCPD
        // TODO: Do not size in GB mode
        self.ocpd.clear();
        self.ocpd.resize(64, 0);

        // Reset: Caches
        self.priority_cache.clear();
        self.priority_cache.resize(WIDTH * HEIGHT, 0);
        self.sprite_x_cache.clear();
        self.sprite_x_cache.resize(WIDTH * HEIGHT, 0);

        // Reset: Registers
        // TODO: Dependent on model/variant
        self.lcd_enable = true;
        self.background_display = true;
        self.tile_data_select = true;
        self.scy = 0;
        self.scx = 0;
        self.lyc = 0;
        self.bgp = 0xFC;
        self.obp0 = 0xFF;
        self.obp1 = 0xFF;
        self.wx = 0;
        self.wy = 0;
        self.window_line = 0;
    }

    /// Read
    pub fn read(&self, address: u16, in_oam_dma: bool) -> u8 {
        match address {
            // Video RAM
            0x8000...0x9FFF => {
                // VRAM cannot be read during mode 3
                if self.lcd_mode != 3 {
                    self.vram[((address & 0x1FFF) + (0x2000 * (self.vram_bank as u16))) as usize]
                } else {
                    0xFF
                }
            }

            // OAM
            0xFE00...0xFE9F => {
                // OAM cannot be read during mode-2 or mode-3
                // OAM cannot be read during OAM DMA
                if !in_oam_dma && self.lcd_mode < 2 {
                    self.oam[(address - 0xFE00) as usize]
                } else {
                    0xFF
                }
            }

            // LCD Control
            0xFF40 => {
                (bits::bit(self.lcd_enable, 7) | bits::bit(self.window_tile_map_select, 6) |
                 bits::bit(self.window_enable, 5) |
                 bits::bit(self.tile_data_select, 4) |
                 bits::bit(self.background_tile_map_select, 3) |
                 bits::bit(self.sprite_16, 2) | bits::bit(self.sprite_enable, 1) |
                 bits::bit(self.background_display, 0))
            }

            // LCDC Status
            0xFF41 => {
                (0x80 | bits::bit(self.lyc_irq_enable, 6) | bits::bit(self.m2_irq_enable, 5) |
                 bits::bit(self.m1_irq_enable, 4) |
                 bits::bit(self.m0_irq_enable, 3) |
                 bits::bit(self.lyc_timer == 0 && (self.ly == self.lyc), 2) |
                 self.lcd_mode)
            }

            // Scroll Y
            0xFF42 => self.scy,

            // Scroll X
            0xFF43 => self.scx,

            // LCDC Y-Coordinate
            0xFF44 => self.ly,

            // LY Compare
            0xFF45 => self.lyc,

            // BG Palette Data
            0xFF47 => self.bgp,

            // Object Palette 0 Palette Data
            0xFF48 => self.obp0,

            // Object Palette 1 Palette Data
            0xFF49 => self.obp1,

            // Window Y Position
            0xFF4A => self.wy,

            // Window X Position (-7)
            0xFF4B => self.wx,

            // VRAM Bank
            0xFF4F if self.mode.unwrap().device() == mode::CGB => self.vram_bank,

            // Background (Color) Palette Index
            0xFF68 if self.mode.unwrap().device() == mode::CGB => {
                (self.bcpi | 0x40 | bits::bit(self.bcpi_ai, 7))
            }

            // Background (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            0xFF69 if self.mode.unwrap().device() == mode::CGB => {
                self.bcpd[self.bcpi as usize] | ((self.bcpi % 2) * 0x80)
            }

            // Object (Color) Palette Index
            0xFF6A if self.mode.unwrap().device() == mode::CGB => {
                (self.ocpi | 0x40 | bits::bit(self.ocpi_ai, 7))
            }

            // Object (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            0xFF6B if self.mode.unwrap().device() == mode::CGB => {
                self.ocpd[self.ocpi as usize] | ((self.ocpi % 2) * 0x80)
            }

            _ => {
                // Unhandled
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8, in_oam_dma: bool) {
        match address {
            // Video RAM
            0x8000...0x9FFF => {
                // TODO: VRAM cannot be written during mode 3
                self.vram[((address & 0x1FFF) + (0x2000 * (self.vram_bank as u16))) as usize] =
                    value
            }

            // OAM
            0xFE00...0xFE9F => {
                // TODO: OAM cannot be written during mode-2 or mode-3
                // OAM cannot be written during OAM DMA
                if !in_oam_dma {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }

            // LCD Control
            0xFF40 => {
                self.lcd_enable = bits::test(value, 7);
                self.window_tile_map_select = bits::test(value, 6);
                self.window_enable = bits::test(value, 5);
                self.tile_data_select = bits::test(value, 4);
                self.background_tile_map_select = bits::test(value, 3);
                self.sprite_16 = bits::test(value, 2);
                self.sprite_enable = bits::test(value, 1);
                self.background_display = bits::test(value, 0);

                // Reset mode/scanline counters on LCD disable
                if !self.lcd_enable {
                    self.ly = 0;
                    self.lcd_mode = 0;
                    self.cycles = 0;
                }
            }

            // LCDC Status
            0xFF41 => {
                self.lyc_irq_enable = bits::test(value, 6);
                self.m2_irq_enable = bits::test(value, 5);
                self.m1_irq_enable = bits::test(value, 4);
                self.m0_irq_enable = bits::test(value, 3);
            }

            // Scroll Y
            0xFF42 => {
                self.scy = value;
            }

            // Scroll X
            0xFF43 => {
                self.scx = value;
            }

            // LY Compare
            0xFF45 => {
                self.lyc = value;
            }

            // BG Palette Data
            0xFF47 => {
                self.bgp = value;
            }

            // Object Palette 0 Palette Data
            0xFF48 => {
                self.obp0 = value;
            }

            // Object Palette 1 Palette Data
            0xFF49 => {
                self.obp1 = value;
            }

            // Window Y Position
            0xFF4A => {
                self.wy = value;
            }

            // Window X Position (-7)
            0xFF4B => {
                self.wx = value;
            }

            // VRAM Bank
            0xFF4F if self.mode.unwrap().device() == mode::CGB => self.vram_bank = value & 1,

            // Background (Color) Palette Index
            0xFF68 if self.mode.unwrap().device() == mode::CGB => {
                self.bcpi = value & 0x3F;
                self.bcpi_ai = bits::test(value, 7);
            }

            // Background (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            0xFF69 if self.mode.unwrap().device() == mode::CGB => {
                self.bcpd[self.bcpi as usize] = value & !((self.bcpi % 2) * 0x80);

                // Auto-increment
                if self.bcpi_ai {
                    self.bcpi += 1;
                    self.bcpi &= 0x3F;
                }
            }

            // Object (Color) Palette Index
            0xFF6A if self.mode.unwrap().device() == mode::CGB => {
                self.ocpi = value & 0x3F;
                self.ocpi_ai = bits::test(value, 7);
            }

            // Object (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            0xFF6B if self.mode.unwrap().device() == mode::CGB => {
                self.ocpd[self.ocpi as usize] = value & !((self.ocpi % 2) * 0x80);

                // Auto-increment
                if self.ocpi_ai {
                    self.ocpi += 1;
                    self.ocpi &= 0x3F;
                }
            }

            _ => {
                // Unhandled
            }
        }
    }

    /// Render (Scanline)
    fn render(&mut self) {
        // TODO: What should the cycle time be when the background is disabled?
        // Rendering a full scanline takes at least 175 cycles
        self.m3_cycles = 175;

        // The PPU stalls for fine SCX adjustments for up to 2 M-Cycles depending on SCX
        let scx_ = self.scx % 8;
        if scx_ == 0 {
            self.m3_cycles += 0;
        } else if scx_ <= 4 {
            self.m3_cycles += 4;
        } else {
            self.m3_cycles += 8;
        }

        // Reset priority caches
        // TODO: Only do this if the background and window will not be drawn for at least 1 pixel
        self.priority_cache.clear();
        self.priority_cache.resize(WIDTH * HEIGHT, 0);
        self.sprite_x_cache.clear();
        self.sprite_x_cache.resize(WIDTH * HEIGHT, 0);

        // TODO: If CGB, background_display just disables priority on background
        if self.lcd_enable && self.background_display {
            self.render_background();
        }

        if self.lcd_enable && self.window_enable {
            self.m3_cycles += self.render_window();
        }

        if self.lcd_enable && self.sprite_enable {
            self.m3_cycles += self.render_sprites();
        }
    }

    /// Render Background (for Scanline)
    fn render_background(&mut self) {
        // TODO: CGB Background Attributes
        // TODO: Generalize so we can use this for both background and window

        // Line (to be rendered); with a high SCY value, the line wraps around
        let line = self.ly.wrapping_add(self.scy);

        // Starting offset to tile map (for this scanline)
        let row = (if self.background_tile_map_select {
            0x1C00
        } else {
            0x1800
        }) + (((line as usize) >> 3) << 5);

        let mut x = self.scx % 8;
        let y = line % 8;
        let offset = (self.ly as usize) * WIDTH as usize;

        for i in 0..(WIDTH as usize) {
            // Offset to the tile index for this 8-pixel spot on the background
            let col = ((self.scx / 8) + (x / 8)) % 32;

            // Get tile index (unsigned)
            let tile_idx = self.get_tile(row, col as usize);

            // Get palette index for tile (given x and y)
            let tile_x = x % 8;
            let tile_y = y;
            let pal_idx = self.get_tile_data(tile_idx, tile_x, tile_y);

            // Set pixel cache
            self.priority_cache[offset + i] = if pal_idx > 0 { 1 } else { 0 };

            // Apply palette to get shade
            let (r, g, b) = self.get_color(pal_idx, self.bgp);

            // Push pixel (color) to framebuffer
            self.framebuffer[((offset + i) * 4)] = b;
            self.framebuffer[((offset + i) * 4) + 1] = g;
            self.framebuffer[((offset + i) * 4) + 2] = r;
            self.framebuffer[((offset + i) * 4) + 3] = 0xFF;

            x += 1;
        }
    }

    /// Render Window (for Scanline)
    fn render_window(&mut self) -> u32 {
        // TODO: CGB Background Attributes
        // TODO: Generalize so we can use this for both background and window

        if self.window_line > 143 {
            return 0;
        }

        if self.wx > 159 {
            return 0;
        }

        if (self.wy > 143) || (self.wy > self.ly) {
            return 0;
        }

        // Rendering the window takes 6 cycles unless WX=0 then it takes 7
        // HACK: Making this take 30 cycles makes Pinball Deluxe run. It's probably incorrect
        //       but I'll leave this in here until @gekkio makes a test that makes it fail
        let mut cycles = 30;
        if self.wx <= 7 {
            cycles += 1;
        }

        // Line (to be rendered)
        let line = self.window_line;
        self.window_line += 1;

        // Starting offset to tile map (for this scanline)
        let row = (if self.window_tile_map_select {
            0x1C00
        } else {
            0x1800
        }) + (((line as usize) >> 3) << 5);

        let mut x = 0;
        let y = line % 8;
        let offset = (self.ly as usize) * WIDTH as usize;

        for i in (if self.wx > 7 {
            (self.wx - 7) as usize
        } else {
            0
        })..(WIDTH as usize) {
            // Offset to the tile index for this 8-pixel spot on the background
            let col = (x / 8) % 32;

            // Get tile index (unsigned)
            let tile_idx = self.get_tile(row, col as usize);

            // Get palette index for tile (given x and y)
            let tile_x = x % 8;
            let tile_y = y;
            let pal_idx = self.get_tile_data(tile_idx, tile_x, tile_y);

            // Set pixel cache
            self.priority_cache[offset + i] = if pal_idx > 0 { 1 } else { 0 };

            // Apply palette to get shade
            let (r, g, b) = self.get_color(pal_idx, self.bgp);

            // Push pixel (color) to framebuffer
            self.framebuffer[((offset + i) * 4)] = b;
            self.framebuffer[((offset + i) * 4) + 1] = g;
            self.framebuffer[((offset + i) * 4) + 2] = r;
            self.framebuffer[((offset + i) * 4) + 3] = 0xFF;

            x += 1;
        }

        cycles
    }

    /// Render Sprites (for Scanline)
    fn render_sprites(&mut self) -> u32 {
        // Sprite attributes reside in the Sprite Attribute Table (
        // OAM - Object Attribute Memory) at $FE00-FE9F.

        // Each of the 40 entries consists of four bytes with the
        // following meanings:
        //  Byte0 - Y Position
        //  Byte1 - X Position
        //  Byte2 - Tile/Pattern Number
        //  Byte3 - Attributes/Flags:
        //    Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
        //           (Used for both BG and Window. BG color 0 is always behind OBJ)
        //    Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
        //    Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
        //    Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
        //    Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
        //    Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)

        let mut cycles = (self.scx & 7) as u32;
        let mut has_sprite_at_0 = false;
        self.sprite_stall_buckets.clear();
        self.sprite_stall_buckets.resize((((168 + self.scx as usize + 7) / 8)), 0);

        let sprite_sz = if self.sprite_16 { 16 } else { 8 };
        let mut n = 0;
        let offset = (self.ly as usize) * WIDTH as usize;

        for i in 0..40 {
            // Gather sprite properties
            let sprite_y = (self.oam[(i * 4)] as i16) - 16;
            let sprite_x = (self.oam[(i * 4) + 1] as i16) - 8;
            let mut sprite_tile = self.oam[(i * 4) + 2] as usize;
            let sprite_attr = self.oam[(i * 4) + 3];

            // Remember, we are rendering on a line-by-line basis
            // Does this sprite intersect our current scanline?

            if (sprite_y <= (self.ly as i16)) && (sprite_y + sprite_sz) > (self.ly as i16) {
                // A maximum of 10 drawn sprites per line are allowed
                if n >= 10 {
                    break;
                }

                // Calculate y-index into the tile (applying y-mirroring)
                let mut tile_y = ((self.ly as i16) - sprite_y) as u8;
                if bits::test(sprite_attr, 6) {
                    tile_y = (sprite_sz as u8) - 1 - tile_y;
                }

                // Sprites can be 8x16 but tiles are only 8x8; adjust sprite_tile and
                // tile_y to reference the upper or lower tile
                if sprite_sz == 16 {
                    if tile_y < 8 {
                        // Top
                        sprite_tile &= 0xFE;
                    } else {
                        // Bottom
                        tile_y -= 8;
                        sprite_tile |= 0x01;
                    }
                }

                // Iterate through the columns of the sprite's tile
                let mut rendered = false;
                for x in 0..8 {
                    // Is this column of the sprite visible on the screen ?
                    if (sprite_x + x >= 0) && (sprite_x + x < (WIDTH as i16)) {
                        let cache_i = ((self.ly as usize) * WIDTH) + (sprite_x + x) as usize;
                        let pcache = self.priority_cache[cache_i];

                        // Another sprite was drawn and the drawn sprite is < on the
                        // X-axis
                        // TODO: only checked in GB mode
                        if bits::test(pcache, 1) &&
                           (self.sprite_x_cache[cache_i] <= (sprite_x + 8) as u8) {
                            continue;
                        }

                        // In CGB mode; there is a override bit that can be set which
                        // forces sprites to bow down to the background layers
                        if bits::test(pcache, 2) {
                            continue;
                        }

                        // Background/Window pixel drawn and sprite flag b7 indicates
                        // that the sprite is behind the background/window
                        if bits::test(pcache, 0) && bits::test(sprite_attr, 7) {
                            continue;
                        }

                        // Calculate the x-index into the tile (applying x-mirroring)
                        let tile_x = (if bits::test(sprite_attr, 5) {
                            (7 - x)
                        } else {
                            x
                        }) as u8;

                        // Get palette index for tile (given x and y)
                        let pal_idx = self.get_tile_data(sprite_tile, tile_x, tile_y);

                        // Update priority cache
                        self.priority_cache[cache_i] |= if pal_idx > 0 { 0x2 } else { 0 };
                        self.sprite_x_cache[cache_i] = (sprite_x + 8) as u8;

                        // Mark this sprite as rendered
                        rendered = true;

                        // Skip if transparent
                        if pal_idx == 0 {
                            continue;
                        }

                        // Apply palette to get shade
                        let (r, g, b) = self.get_color(pal_idx,
                                                       if bits::test(sprite_attr, 4) {
                                                           self.obp1
                                                       } else {
                                                           self.obp0
                                                       });

                        // Push pixel (color) to framebuffer
                        self.framebuffer[((offset + (sprite_x + x) as usize) * 4)] = b;
                        self.framebuffer[((offset + (sprite_x + x) as usize) * 4) + 1] = g;
                        self.framebuffer[((offset + (sprite_x + x) as usize) * 4) + 2] = r;
                        self.framebuffer[((offset + (sprite_x + x) as usize) * 4) + 3] = 0xFF;
                    } else {
                        // TODO: Check this; should a non-visible sprite far off on the X-axis
                        //       be counted as rendered?
                        //       According to various docs it should.. but.. we all know how
                        //       accurate those can be.
                        rendered = true;
                    }
                }

                if rendered {
                    let sprite_x = self.oam[(i * 4) + 1] as i16;
                    if sprite_x < 168 {
                        // Visible and rendered sprite affects timing
                        // This is a visible sprite; takes 6 cycles
                        cycles += 6;

                        let mut x = sprite_x + (self.scx as i16);

                        if x < 0 {
                            x = 0;
                        }

                        // Mark if this sprite is at <=0
                        if x <= 0 {
                            has_sprite_at_0 = true;
                        }

                        // Determine and update stall bucket
                        //  Each sprite drawn causes a stall of up to 5 cycles
                        //  in each 8-pixel "bucket".
                        let bucket_i = (x >> 3) as usize;
                        let mut stall = 5 - (x & 7);
                        if stall < 0 {
                            stall = 0;
                        }

                        self.sprite_stall_buckets[bucket_i] =
                            cmp::max(self.sprite_stall_buckets[bucket_i], stall as u8);
                    }

                    // Rendered sprite affects max # of sprites per scanline
                    n += 1;
                }
            }
        }

        // Sum the 8-pixel bucket stalls
        for &b in &self.sprite_stall_buckets {
            cycles += b as u32;
        }

        // If a sprite is at x<=0; PPU stalls for an additional SCX & 7
        if has_sprite_at_0 {
            cycles += (self.scx & 7) as u32;
        }

        // Floor T-cycles to the lowest M-cycle (extra cycles get chopped off)
        (cycles >> 2) << 2
    }

    /// Get tile index from background tile map
    fn get_tile(&self, row: usize, column: usize) -> usize {
        if self.tile_data_select {
            self.vram[row + column] as usize
        } else {
            (((self.vram[row + column] as i8) as isize) + 256) as usize
        }
    }

    /// Get tile data (palette index)
    fn get_tile_data(&self, tile: usize, x: u8, y: u8) -> u8 {
        let offset = tile * 16 + ((y as usize) * 2);

        (((self.vram[offset + 1] >> (7 - x) << 1) & 2) | (self.vram[offset] >> (7 - x) & 1))
    }

    /// Get color from palette and palette index
    // TODO: Make configurable
    fn get_color(&self, palette_index: u8, palette: u8) -> (u8, u8, u8) {
        let pixel = (palette >> (palette_index << 1)) & 0x3;
        if pixel == 1 {
            // Grayscale
            (0xC0, 0xC0, 0xC0)
            // Green
            // 0xFF8BB30F
            // Yellow
            // 0xFFABA92F
        } else if pixel == 2 {
            // Grayscale
            (0x60, 0x60, 0x60)
            // Green
            // 0xFF306230
            // Yellow
            // 0xFF565413
        } else if pixel == 3 {
            // Grayscale
            (0, 0, 0)
            // Green
            // 0xFF0F410F
            // Yellow
            // 0xFF000000
        } else {
            // Grayscale
            (0xFF, 0xFF, 0xFF)
            // Green
            // 0xFF9BBC0F
            // Yellow
            // 0xFFFFFD4B
        }
    }
}
