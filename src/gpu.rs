use std::vec::Vec;

use ::bits;

/// Display width
const WIDTH: usize = 160;

/// Display height
const HEIGHT: usize = 144;

#[derive(Default)]
pub struct GPU {
    // TODO: This should not be public but it is for my hacked SDL usage
    /// Pixel data (for current "frame") buffer that is rewritten line-by-line during H-Blank
    pub framebuffer: Vec<u8>,

    /// [0x8000 - 0x9FFF] Video RAM (VRAM) — 8 KiB x 2 Banks (1 Bank in GB)
    vram: Vec<u8>,

    /// [0xFF4F] Video RAM Bank (VBK) 0 - 1 (R/W; CGB only)
    vram_bank: u8,

    /// [0xFE00 — 0xFE9F] Sprite Attribute Table (OAM) — 160 Bytes
    oam: Vec<u8>,

    /// STAT IRQ; current value
    /// Actual interrupt is triggered when this goes from 0 to 1
    stat_irq: bool,

    /// T-Cycle counter for GPU mode
    cycles: u32,

    /// T-Cycle counter for Mode 3 (variable length)
    m3_cycles: u32,

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
    mode: u8,

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
    /// Step
    pub fn step(&mut self, if_: &mut u8) {
        // The machine is stepped each M-cycle and the GPU needs to be stepped each T-cycle
        for _ in 1..5 {
            // TODO: What do we do when the LCD is disabled?
            if !self.lcd_enable {
                break;
            }

            // Mode 2 and 0 IRQ are raised some cycles before the GPU actually enters the
            // respective modes; we use this variable to mark the mode being compared
            // when we check for a IRQ. If it is still 0xFF it gets set to `self.mode`.
            let mut mode_cmp = 0xFF;

            // LY is compared to LYC and the "coincidence" IRQ is raised. However, when the
            // LY register is updated, there is a 4 T-Cycle delay until it can be compared.
            if self.lyc_timer > 0 {
                self.lyc_timer -= 1;
            }

            // Increment cycle counter (reset to 0 at the beginning of each scanline)
            self.cycles += 1;

            if self.mode == 0 && self.cycles == 5 {
                // Lines 0-144 start each scanline in mode 0 for 4 cycles
                if self.ly < 144 {
                    // Proceed to mode 2 — Searching OAM-RAM
                    self.mode = 2;
                } else {
                    // Proceed to mode 1 — V-Blank
                    self.mode = 1;

                    // Trigger VBL interrupt
                    (*if_) |= 0x1;

                    // TODO: Trigger the front-end to refresh the scren
                }
            } else if self.mode == 0 && self.cycles >= 1 && self.cycles < 5 && self.ly >= 1 &&
                      self.ly <= 143 {
                // Lines 1-143 signal the M2 IRQ 4 T-cycles early
                mode_cmp = 2;
            } else if self.mode == 2 && self.cycles == 85 {
                // Proceed to mode 3 — Transferring Data to LCD Driver
                self.mode = 3;

                // Render scanline (at the -start- of mode-3)
                self.render();
            } else if self.mode == 3 && self.cycles >= ((85 + self.m3_cycles) - 7) &&
                      self.cycles < (85 + self.m3_cycles) {
                // The mode M0 IRQ is signalled 7 T-cycles early
                // TODO: Try and verify that
                mode_cmp = 0;
            } else if self.mode == 3 && self.cycles == (85 + self.m3_cycles) {
                // Proceed to mode 0 — H-Blank
                self.mode = 0;
            } else if self.mode == 0 && self.cycles == 457 {
                // A scanline takes 456 T-Cycles to complete
                // This acts as the 0th scanline for lines 1-143
                self.ly += 1;
                self.lyc_timer = 4;
                self.mode = 0;
                self.cycles = 1;
            } else if self.mode == 1 {
                if self.cycles == 457 {
                    self.cycles = 1;

                    if self.ly == 0 {
                        // Restart process (back to top of LCD)
                        self.mode = 0;
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
                mode_cmp = self.mode;
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
    }

    /// Reset
    pub fn reset(&mut self) {
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
    }

    /// Read
    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Video RAM
            0x8000...0x9FFF => {
                // TODO: VRAM cannot be read during mode 3
                self.vram[((address & 0x1FFF) + (0x2000 * (self.vram_bank as u16))) as usize]
            }

            // OAM
            0xFE00...0xFE9F => {
                // TODO: OAM cannot be read during mode-2 or mode-3
                // TODO: OAM cannot be read during OAM DMA
                self.oam[(address - 0xFE00) as usize]
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
                 self.mode)
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
            // TODO: Only accessible in CGB
            0xFF4F => self.vram_bank,

            // Background (Color) Palette Index
            // TODO: Only accessible in CGB
            0xFF68 => (self.bcpi | 0x40 | bits::bit(self.bcpi_ai, 7)),

            // Background (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            // TODO: Only accessible in CGB
            0xFF69 => self.bcpd[self.bcpi as usize] | ((self.bcpi % 2) * 0x80),

            // Object (Color) Palette Index
            // TODO: Only accessible in CGB
            0xFF6A => (self.ocpi | 0x40 | bits::bit(self.ocpi_ai, 7)),

            // Object (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            // TODO: Only accessible in CGB
            0xFF6B => self.ocpd[self.ocpi as usize] | ((self.ocpi % 2) * 0x80),

            _ => {
                // Unhandled
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
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
                // TODO: OAM cannot be written during OAM DMA
                self.oam[(address - 0xFE00) as usize] = value;
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
                    self.mode = 0;
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
            // TODO: Only accessible in CGB
            0xFF4F => self.vram_bank = value & 1,

            // Background (Color) Palette Index
            // TODO: Only accessible in CGB
            0xFF68 => {
                self.bcpi = value & 0x3F;
                self.bcpi_ai = bits::test(value, 7);
            }

            // Background (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            // TODO: Only accessible in CGB
            0xFF69 => {
                self.bcpd[self.bcpi as usize] = value & !((self.bcpi % 2) * 0x80);

                // Auto-increment
                if self.bcpi_ai {
                    self.bcpi += 1;
                    self.bcpi &= 0x3F;
                }
            }

            // Object (Color) Palette Index
            // TODO: Only accessible in CGB
            0xFF6A => {
                self.ocpi = value & 0x3F;
                self.ocpi_ai = bits::test(value, 7);
            }

            // Object (Color) Palette Data
            //  Every 2nd byte has 1 unused bit
            // TODO: Only accessible in CGB
            0xFF6B => {
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
        if scx_ <= 4 {
            self.m3_cycles += 4;
        } else {
            self.m3_cycles += 8;
        }

        // TODO: If CGB, background_display just disables priority on background
        if self.lcd_enable && self.background_display {
            self.render_tiles();
        }
    }

    /// Render Tile Map (for Scanline)
    fn render_tiles(&mut self) {
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
