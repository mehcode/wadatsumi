// Copyright (C) 2026 Ryan Leckey <leckey.ryan@gmail.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Parity of the current frame.
///
/// The PPU shaves one dot off the pre-render scanline on every odd frame when background
/// rendering is enabled, jumping straight from `(scanline 261, dot 339)` to `(0, 0)`
/// instead of running dot 340. That half-CPU-cycle shift interleaves the raster against
/// the NTSC color subcarrier so hue stays stable from one frame to the next. Stored as
/// its own enum (rather than a `bool`) because `Even` / `Odd` reads cleanly at the
/// dot-skip site, and because the bottom bit of the absolute frame count is the only
/// thing the dot-skip logic actually consults.
///
/// See <https://www.nesdev.org/wiki/PPU_frame_timing>.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuFrameParity {
    /// Even-numbered frame. The pre-render scanline always runs the full 341 dots.
    Even,

    /// Odd-numbered frame. The pre-render scanline runs only 340 dots when background
    /// rendering is enabled at the `(261, 339)` boundary; otherwise it runs the full 341.
    Odd,
}

/// The PPU's position within the current frame: which dot of which scanline is being
/// processed, plus the parity bit and absolute frame count needed to drive the NTSC
/// pre-render dot-skip and host frame-completion callbacks.
///
/// Advanced one dot per call to [`Ppu2C02::tick`](crate::Ppu2C02::tick) (three dots per
/// CPU cycle on NTSC). The tick state machine reads this to know when to set the vblank
/// flag (scanline 241, dot 1), when to clear the status bits at the start of pre-render
/// (scanline 261, dot 1), and which background/sprite fetch is due on each dot.
///
/// See <https://www.nesdev.org/wiki/PPU_frame_timing> for the full dot timeline.
///
#[derive(Debug, Clone, Copy)]
pub struct PpuFrameClock {
    /// Horizontal dot within the scanline (`0..=340`).
    ///
    /// Within a visible scanline the dots break down as:
    ///
    /// - `0`: idle, no memory access
    ///
    /// - `1..=256`: background fetches and pixel output for the 256 visible pixels
    ///
    /// - `257..=320`: sprite fetches for the *next* scanline
    ///
    /// - `321..=336`: prefetch the first two tiles of the next scanline
    ///
    /// - `337..=340`: two extra dummy nametable fetches (used by MMC5 for scanline
    ///   counting)
    ///
    /// `u16` rather than `u8` so the upper bound and arithmetic stay cast-free.
    ///
    pub dot: u16,

    /// Vertical scanline within the frame (`0..=261`).
    ///
    /// - `0..=239`: visible scanlines; pixels actually leave the chip
    ///
    /// - `240`: post-render scanline; PPU idles, vblank flag is *not* yet set
    ///
    /// - `241..=260`: vertical blank; CPU has uninterrupted VRAM/OAM access. Vblank flag
    ///   is set at `(241, 1)` and NMI fires there if `PPUCTRL` bit 7 is on
    ///
    /// - `261`: pre-render scanline; status flags are cleared at dot 1 and the first
    ///   tiles of the next frame are prefetched
    ///
    /// Matched in width to [`Self::dot`] so comparisons against `241` / `261` don't need
    /// casts.
    ///
    pub scanline: u16,

    /// Parity of the current frame, toggled every time `scanline` wraps `261` → `0`.
    ///
    /// Consulted by the dot-skip at `(261, 339)`; see [`PpuFrameParity`] for the *why*.
    /// Tracked separately from [`Self::count`] because the dot-skip only cares about the
    /// bottom bit and `count` may roll over (in 2^64 frames, but still).
    ///
    pub parity: PpuFrameParity,

    /// Monotonic frame counter, bumped each time `scanline` wraps `261` → `0`.
    ///
    /// Not consulted by the timing state machine itself; provided for host frame-completion
    /// callbacks (so the runtime can blit a finished frame) and deterministic test
    /// assertions of the form "at frame N, scanline 241, dot 1, vblank is set."
    ///
    pub count: u64,
}

impl Default for PpuFrameClock {
    fn default() -> Self {
        Self::new()
    }
}

impl PpuFrameClock {
    /// Constructs a clock parked at frame 0, scanline 0, dot 0, even parity.
    #[must_use]
    pub const fn new() -> Self {
        Self { dot: 0, scanline: 0, parity: PpuFrameParity::Even, count: 0 }
    }

    /// Advances the clock by one dot.
    ///
    /// Steps `dot` through `0..=340`, wrapping into the next scanline at 341, and wraps
    /// `scanline` through `0..=261` back into the next frame at 262. Each frame wrap
    /// bumps `count` and flips `parity`.
    ///
    /// Pass `PPUMASK`'s `rendering_enabled` (background OR sprites enabled) at the moment
    /// of the call. Sampling at call time matches hardware: turning rendering off between
    /// dots 339 and 340 of an odd pre-render scanline cancels the skip for that frame.
    ///
    pub const fn advance(&mut self, rendering_enabled: bool) {
        // NTSC pre-render dot-skip. On odd frames with rendering on, the PPU jumps
        // straight from (261, 339) to (0, 0) of the next (even) frame, skipping dot 340
        // entirely. Handled as a special case before the normal step so the regular
        // dot/scanline math never sees the elided dot. We know the outgoing parity is
        // Odd here, so the incoming parity is unconditionally Even.
        if rendering_enabled
            && matches!(self.parity, PpuFrameParity::Odd)
            && self.scanline == 261
            && self.dot == 339
        {
            self.dot = 0;
            self.scanline = 0;
            self.count = self.count.wrapping_add(1);
            self.parity = PpuFrameParity::Even;

            return;
        }

        // Step one dot within the current scanline. Most ticks take this path and exit
        // immediately; the wraps below only run on the last dot of a scanline / frame.
        self.dot += 1;
        if self.dot <= 340 {
            return;
        }

        // Scanline wrap. Dot just rolled past 340, so we're at the start of the next
        // scanline. Frames are 0..=261, so anything still in-range here is mid-frame.
        self.dot = 0;
        self.scanline += 1;
        if self.scanline <= 261 {
            return;
        }

        // Frame wrap. Scanline just rolled past 261, so we're at (0, 0) of a fresh
        // frame. Bump the absolute counter and flip parity so the next pre-render dot
        // 339 knows whether to skip dot 340 above.
        self.scanline = 0;
        self.count = self.count.wrapping_add(1);
        self.parity = match self.parity {
            PpuFrameParity::Even => PpuFrameParity::Odd,
            PpuFrameParity::Odd => PpuFrameParity::Even,
        };
    }
}
