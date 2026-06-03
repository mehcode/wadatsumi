use std::path::Path;

use wadatsumi::{Bus, System};

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

fn instr_test_v5(path: &Path) -> datatest_stable::Result<()> {
    let mut system = System::new();
    system.open(path)?;

    let mut initialized = false;

    for _ in 0..MAX_CYCLES {
        system.tick()?;

        // The ROM writes a 3-byte magic signature to $6001-$6003 once its startup
        // routine finishes. We ignore $6000 until we see it, before then the SRAM
        // is zero-initialized and $6000 would be a false "pass".
        if !initialized {
            initialized = system.bus.peek(0x6001) == 0xDE
                && system.bus.peek(0x6002) == 0xB0
                && system.bus.peek(0x6003) == 0x61;

            continue;
        }

        // $80 is the "still running" sentinel; any other value means the ROM
        // has finished and written its final result code to $6000.
        let status = system.bus.peek(0x6000);

        if status == 0x80 {
            continue;
        }

        // A non-zero result is a failure. The ROM also writes a human-readable
        // description to $6004 as a null-terminated ASCII string.
        if status != 0x00 {
            let msg: String = (0x6004..)
                .map(|a| system.bus.peek(a))
                .take_while(|&b| b != 0)
                .map(|b| b as char)
                .collect();

            return Err(format!("status={status:#04x}\n{msg}").into());
        }

        return Ok(());
    }

    Err(format!("timed out after {MAX_CYCLES} cycles (initialized={initialized})").into())
}
