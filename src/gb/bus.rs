use std::vec::Vec;

use gb::cart;
use ::mode;
use gb::gpu;
use gb::apu;
use gb::timer;
use gb::joypad;

/// The Bus is the interconnect that facilitates communication from the CPU and the various other
/// components in the machine.
#[derive(Default)]
pub struct Bus {
    /// Component: Cartridge (Reader)
    pub cart: cart::Cartridge,

    /// Component: GPU
    pub gpu: gpu::GPU,

    /// Component: APU
    pub apu: apu::APU,

    /// Component: Timer
    pub timer: timer::Timer,

    /// Component: Joypad
    pub joypad: joypad::Joypad,

    /// [0xC000 - 0xDFFF] Work RAM (WRAM)
    ///   8 KiB in GB
    ///  32 KiB in CGB
    pub wram: Vec<u8>,

    /// [0xFF70] WRAM Bank 0 - 7 (SVBK) — CGB Only
    pub wram_bank: u8,

    /// [0xFF80 - 0xFFFE] High RAM (HRAM) — 127 Bytes
    pub hram: Vec<u8>,

    /// [0xFFFF] Interrupt Enable (IE) R/W
    pub ie: u8,

    /// [0xFF0F] Interrupt Flag (IF) R/W
    pub if_: u8,

    /// [OAM DMA] Source Address for the running OAM DMA
    oam_dma_source: u16,

    /// [OAM DMA] Source Address for the next OAM DMA (scheduled by `_delay_timer`)
    oam_dma_next_source: u16,

    /// [OAM DMA] Delay (in M-cycles) until the next OAM DMA begins
    oam_dma_delay_timer: u8,

    /// [OAM DMA] Index into the running OAM DMA
    oam_dma_index: u16,

    /// [OAM DMA] Timer (in M-Cycles) of how long we have left in OAM DMA
    oam_dma_timer: u16,
}

impl Bus {
    /// Step
    pub fn step(&mut self) {
        // [OAM DMA] Run next iteration (if active)
        if self.oam_dma_timer > 0 {
            // Each tick does a single byte memory copy
            let src = self.oam_dma_source + self.oam_dma_index;
            let r = self.read(src);
            // info!("oam/transfer [{:X}] {:X} -> [{:X}]",
            //       src,
            //       r,
            //       0xFE00 + self.oam_dma_index);
            self.gpu.oam[self.oam_dma_index as usize] = r;

            self.oam_dma_index += 1;
            self.oam_dma_timer -= 1;
        }

        // When OAM DMA starts a delay timer is set to 2; the tick with the memory
        // write that starts DMA and the tick just after are wait cycles before
        // the actual DMA starts. If there was an existing DMA running; that DMA
        // does not stop until the next one starts
        if self.oam_dma_delay_timer > 0 {
            self.oam_dma_delay_timer -= 1;
            if self.oam_dma_delay_timer == 0 {
                self.oam_dma_timer = 160;
                self.oam_dma_index = 0;
                self.oam_dma_source = self.oam_dma_next_source;
            }
        }

        // The Bus is stepped by the CPU each M-cycle and it must then step the system
        // components 4 T-cycles
        for _ in 0..4 {
            // TODO(architecture): This feels _wrong_ but the GPU and Timer need IF to signal IRQ
            //      Perhaps make a separate IRQ subsystem that is given out here?

            self.timer.step();

            let div_last = self.timer.div_last;
            let div = self.timer.div;

            self.timer.on_change_div(div_last, div, &mut self.if_);

            self.gpu.step(&mut self.if_);

            self.apu.step();
            self.apu.on_change_div(self.timer.div_last, self.timer.div);
        }
    }

    /// Reset
    pub fn reset(&mut self, mode: mode::Mode) {
        // Interrupt Enable/Flag
        self.ie = 0;
        self.if_ = 0x1;

        // Reset: WRAM
        self.wram.clear();
        // TODO: Depends on model (gb/cgb)
        self.wram.resize(32 * 1024, 0);

        // Reset: HRAM
        self.hram.clear();
        self.hram.resize(127, 0);

        // Reset: Components
        self.cart.reset();
        self.gpu.reset(mode);
        self.apu.reset();
        self.joypad.reset();
        self.timer.reset(mode);

        // Reset: OAM DMA
        self.oam_dma_source = 0;
        self.oam_dma_next_source = 0;
        self.oam_dma_delay_timer = 0;
        self.oam_dma_index = 0;
        self.oam_dma_timer = 0;
    }

    /// Read
    pub fn read(&mut self, address_: u16) -> u8 {
        let mut address = address_;

        // During an OAM DMA; accesses to memory outside of HIRAM are essentially wonky
        // If there is a bus conflict (eg. CPU is accessing memory where the OAM DMA process
        // wants to access), the OAM DMA process wins and the CPU sees the value the OAM DMA
        // process just read.
        if self.oam_dma_timer > 0 {
            let ext_bus_1 = 0x0000..0x8000;
            let ext_bus_2 = 0xA000..0xFE00;
            let vram_bus = 0x8000..0xA000;

            if (ext_bus_1.contains(self.oam_dma_source) && ext_bus_1.contains(address)) ||
               (ext_bus_2.contains(self.oam_dma_source) && ext_bus_2.contains(address)) ||
               (vram_bus.contains(self.oam_dma_source) && vram_bus.contains(address)) {
                address = self.oam_dma_source + self.oam_dma_index;
            }
        }

        match address {
            // Cartridge
            0x0000...0x7FFF | 0xA000...0xBFFF => self.cart.read(address),

            // Video RAM, OAM, GPU registers
            0x8000...0x9FFF | 0xFE00...0xFE9F | 0xFF40...0xFF45 | 0xFF47...0xFF4F |
            0xFF68...0xFF6B => self.gpu.read(address, self.oam_dma_timer != 0),

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)]
            }

            // Joypad
            0xFF00 => self.joypad.read(address),

            // Serial Data Transfer (Link Cable)
            // TODO: Not implemented yet; just the boot values returned here
            0xFF01 => 0,
            0xFF02 => 0x7E,

            // Timer
            0xFF04...0xFF07 => self.timer.read(address),

            // APU
            0xFF10...0xFF3F => self.apu.read(address),

            // High RAM
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],

            // Interrupt Flag (IF)
            0xFF0F => (self.if_ | 0xE0),

            // Interrupt Enable (IE)
            0xFFFF => self.ie,

            _ => {
                // Unhandled
                // warn!("unhandled read at {:#04X}", address);
                0xFF
            }
        }
    }

    /// Write
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Cartridge
            0x0000...0x7FFF | 0xA000...0xBFFF => self.cart.write(address, value),

            // OAM DMA
            0xFF46 => {
                // DMA - DMA Transfer and Start Address (W)
                self.oam_dma_next_source = (value as u16) << 8;
                self.oam_dma_delay_timer = 2;
            }

            // Video RAM, OAM, GPU registers
            0x8000...0x9FFF | 0xFE00...0xFE9F | 0xFF40...0xFF45 | 0xFF47...0xFF4F |
            0xFF68...0xFF6B => {
                self.gpu.write(address, value, self.oam_dma_timer != 0);
            }

            // Work RAM
            0xC000...0xFDFF => {
                self.wram[(address as usize & 0x1FFF) + (self.wram_bank as usize * 0x2000)] = value;
            }

            // Joypad
            0xFF00 => self.joypad.write(address, value),

            // Timer
            0xFF04...0xFF07 => {
                let div = self.timer.div;
                self.timer.write(address, value, &mut self.if_);

                // TODO(architecture): I can't think of a better way to observe DIV changes from
                //  outside of the timer system
                if div != self.timer.div {
                    let div_last = self.timer.div_last;
                    let div = self.timer.div;

                    self.timer.on_change_div(div_last, div, &mut self.if_);
                    self.apu.on_change_div(div_last, div);
                }
            }

            // APU
            0xFF10...0xFF3F => self.apu.write(address, value),

            // High RAM
            0xFF80...0xFFFE => {
                self.hram[(address - 0xFF80) as usize] = value;
            }

            // Interrupt Flag (IF)
            0xFF0F => {
                self.if_ = value & !0xE0;
            }

            // Interrupt Enable (IE)
            0xFFFF => {
                self.ie = value;
            }

            _ => {
                // Unhandled
                // warn!("unhandled write at {:#04X} with {:#02X}", address, value);
            }
        }
    }
}
