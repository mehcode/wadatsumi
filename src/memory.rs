pub struct Memory {
    // [02000000 - 0203FFFF] WRAM - On-board Work RAM  (256 KBytes) 2 Wait
    work_ram_1: Box<[u8]>,

    // [03000000 - 03007FFF] WRAM - In-chip Work RAM   (32 KBytes)
    work_ram_2: Box<[u8]>,

    // [05000000 - 050003FF] BG/OBJ Palette RAM        (1 Kbyte)
    palette_ram: Box<[u8]>,

    // [06000000 - 06017FFF] VRAM - Video RAM          (96 KBytes)
    v_ram: Box<[u8]>,

    // [07000000 - 070003FF] OAM - OBJ Attributes      (1 Kbyte)
    oam: Box<[u8]>,

    // [08000000 - 09FFFFFF] Game Pak ROM/FlashROM (max 32MB) - Wait State 0
    // [0A000000 - 0BFFFFFF] Game Pak ROM/FlashROM (max 32MB) - Wait State 1
    // [0C000000 - 0DFFFFFF] Game Pak ROM/FlashROM (max 32MB) - Wait State 2
    rom: Box<[u8]>,

    // [0E000000 - 0E00FFFF] Game Pak SRAM    (max 64 KBytes) - 8bit Bus width
    s_ram: Box<[u8]>,
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            work_ram_1: vec![0; 256 * 1024].into_boxed_slice(),
            work_ram_2: vec![0; 32 * 1024].into_boxed_slice(),
            palette_ram: vec![0; 1024].into_boxed_slice(),
            v_ram: vec![0; 96 * 1024].into_boxed_slice(),
            oam: vec![0; 1024].into_boxed_slice(),
            rom: rom.into_boxed_slice(),
            s_ram: vec![64; 1024].into_boxed_slice(),
        }
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        match address {
            0x02000000..=0x0203FFFF => self.work_ram_1[(address - 0x02000000) as usize],
            0x03000000..=0x03007FFF => self.work_ram_2[(address - 0x03000000) as usize],

            // Mirror of 0x03_FF_FF_xx
            0x03FFFF00..=0x03FFFFFF => self.work_ram_2[((address - 0x03FFFF00) + 0x7F00) as usize],

            0x08000000..=0x0DFFFFFF => self.rom[(address - 0x08000000) as usize],

            _ => {
                unimplemented!("read from unhandled address 0x{:x}", address)
            }
        }
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        *(match address {
            0x02000000..=0x0203FFFF => &mut self.work_ram_1[(address - 0x02000000) as usize],
            0x03000000..=0x03007FFF => &mut self.work_ram_2[(address - 0x03000000) as usize],

            // Mirror of 0x03_FF_FF_xx
            0x03FFFF00..=0x03FFFFFF => &mut self.work_ram_2[((address - 0x03FFFF00) + 0x7F00) as usize],

            0x08000000..=0x0DFFFFFF => &mut self.rom[(address - 0x08000000) as usize],

            _ => {
                unimplemented!("write to unhandled address 0x{:x}", address)
            }
        }) = value;
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        // todo: if we are in a 32-bit data bus, maybe we can read the whole 32-bits at once
        let a = self.read_u8(address) as u32;
        let b = self.read_u8(address + 1) as u32;
        let c = self.read_u8(address + 2) as u32;
        let d = self.read_u8(address + 3) as u32;

        let value = a | (b << 8) | (c << 16) | (d << 24);

        log::trace!("read [0x{:x}] -> 0x{:08x} (u32)", address, value);

        value
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        log::trace!("write [0x{:x}] <- 0x{:08x} (u32)", address, value);

        self.write_u8(address, value as u8);
        self.write_u8(address + 1, (value >> 8) as u8);
        self.write_u8(address + 2, (value >> 16) as u8);
        self.write_u8(address + 3, (value >> 24) as u8);
    }
}
