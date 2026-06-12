// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Side-effect-free read access to the CPU address bus, for debuggers and
/// disassemblers that want to look at memory without disturbing it.
///
/// Reading some addresses on a real NES has side effects: `$2002` clears the vblank
/// flag and the `w` latch, `$2007` advances the VRAM pointer, `$4015` clears the APU
/// frame-counter IRQ. [`CpuReadWrite::read`] applies those side effects because the
/// CPU has to; [`CpuPeek::peek`] skips them so an observer doesn't perturb the game
/// while it's running.
///
pub trait CpuPeek {
    /// Reads one byte from `address` without triggering hardware side effects.
    ///
    /// Where [`CpuReadWrite::read`] would clear a flag or advance an internal
    /// register, this returns the value the CPU *would* have seen and leaves the
    /// underlying state alone.
    ///
    fn peek(&self, address: u16) -> u8;
}

/// Read/write access to the CPU address bus.
///
/// The CPU talks to the rest of the system through this trait: WRAM at
/// `$0000`..`$1FFF`, PPU registers at `$2000`..`$3FFF`, APU and I/O registers at
/// `$4000`..`$401F`, optional pak SRAM at `$6000`..`$7FFF`, and PRG-ROM at
/// `$8000`..`$FFFF`. The implementor handles mirroring, open-bus, and the
/// per-region side effects; the CPU itself just hands a byte off and trusts the
/// bus to route it.
///
/// This is the narrow trait, just the address/data bus. Interrupt and `RDY`
/// observation ride on [`CpuBus`], which extends this one.
///
pub trait CpuReadWrite {
    /// Reads one byte from `address`, applying any hardware side effects.
    ///
    /// Reading `$2002` clears the vblank flag and the `w` latch, `$2007` advances
    /// `v` by the configured `PPUDATA` increment, `$4015` clears the APU frame
    /// IRQ, and open-bus addresses return whatever the data lines were floating
    /// at. Use [`CpuPeek::peek`] when you want the value without the side effect.
    ///
    fn read(&mut self, address: u16) -> u8;

    /// Writes `value` to `address`, applying any hardware side effects.
    ///
    /// PPU register writes update internal state immediately (`$2000` changing the
    /// base nametable, `$2006` shifting the address latch, and so on), `$4014`
    /// kicks off OAM DMA, and mapper register writes (`$8000`..`$FFFF` on most
    /// boards) reconfigure bank switching. The CPU doesn't care which is which;
    /// it just drops the byte on the bus.
    ///
    fn write(&mut self, address: u16, value: u8);
}

/// Everything the CPU observes from its place on the board: the address/data bus
/// from [`CpuReadWrite`], plus the `/NMI`, `/IRQ`, and `RDY` control lines.
///
/// "Bus" here follows emulator-vernacular usage, the whole system interface the
/// CPU sees, not the strict definition of address + data only. Splitting it
/// from [`CpuReadWrite`] keeps non-CPU consumers (debugger memory pokes, future
/// tooling) on the narrow trait so they don't have to think about control lines.
///
/// All three line accessors are modeled active-high (`true` means the signal is
/// being asserted) even though the real `/NMI` and `/IRQ` pins are active-low. We
/// sample the asserter's intent rather than the wire voltage, which spares every
/// implementor from inverting at the bus boundary.
///
/// The accessors all default to "inactive" so a test bus only needs a trivial
/// `impl CpuBus for TestBus {}` to opt in.
///
/// See <https://www.nesdev.org/wiki/CPU_interrupts> for the canonical reference
/// on how the 6502 services NMI and IRQ.
///
pub trait CpuBus: CpuReadWrite {
    /// Current level of the `/NMI` line: `true` while something is asserting NMI.
    ///
    /// The CPU samples this every cycle and latches the rising edge internally;
    /// the latched bit is what actually triggers an NMI sequence at the next
    /// instruction boundary, not the level itself. So a one-cycle pulse is enough
    /// to fire one NMI, and holding NMI high indefinitely still only fires once
    /// per assertion.
    ///
    /// On the NES the PPU drives this as `vblank_flag AND PPUCTRL bit 7`, so
    /// flipping the enable bit on while vblank is still up re-fires NMI
    /// mid-frame, the "multi-NMI" trick some games lean on.
    ///
    fn nmi(&self) -> bool {
        false
    }

    /// Current level of the `/IRQ` line: `true` while something is asserting IRQ.
    ///
    /// Level-sampled every cycle and gated by the `I` flag at the fetch boundary,
    /// so the asserter has to keep driving the line until the handler
    /// acknowledges (typically by writing the source's ack register). Drop the
    /// line too early and the CPU may miss the request entirely.
    ///
    /// On the NES this is the wired-OR of the mapper's IRQ output (MMC3, MMC5,
    /// FME-7, VRC4/6/7) and the APU's frame-counter and DMC IRQ outputs. The bus
    /// implementor is responsible for combining them.
    ///
    fn irq(&self) -> bool {
        false
    }

    /// Current level of the `RDY` line: `true` when the CPU is free to run,
    /// `false` while a DMA controller is holding the bus.
    ///
    /// The CPU only checks `RDY` on read cycles. When it's low the read cycle
    /// stretches indefinitely, no PC advance and no bus access, and writes go
    /// through unaffected. That matches the real chip, where DMA controllers can
    /// only steal the bus on a read.
    ///
    /// On the NES the 2A03's DMA controllers pull this low to steal cycles: OAM
    /// DMA (`$4014`) for 513 or 514 cycles to stream OAM, and DMC DMA for 1 to 4
    /// cycles when the sample fetcher needs the bus.
    ///
    fn rdy(&self) -> bool {
        true
    }
}
