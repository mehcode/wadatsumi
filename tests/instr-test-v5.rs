use std::path::Path;

use wadatsumi::System;

datatest_stable::harness! {
    {
        test = instr_test_v5,
        root = "tests/instr-test-v5",
        pattern = r"^.*\.nes$",
    },
}

// Cycle budget before we declare a hang. At ~1.79 MHz the slowest sub-test
// completes well under 100 M cycles.
const MAX_CYCLES: u64 = 100_000_000;

/// Smoke-tests each instr_test_v5 ROM by running it for up to [`MAX_CYCLES`] without crashing.
/// Once `Bus::peek` is available, this should poll `$6000` for the blargg result code and
/// assert it is zero, with the diagnostic string from `$6004` on failure.
fn instr_test_v5(path: &Path) -> datatest_stable::Result<()> {
    let mut system = System::new();
    system.open(path)?;

    for _ in 0..MAX_CYCLES {
        system.tick()?;

        // TODO: monitor $6000 for test completion
        // TODO: read status code from $6004..
    }

    Ok(())
}
