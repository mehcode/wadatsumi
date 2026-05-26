use wadatsumi::{Bus, System};

/// The canonical NES CPU conformance ROM, authored by kevtris.
///
/// The ROM exhaustively exercises every official 2A03 opcode across all
/// addressing modes, verifying cycle counts, flag effects, and memory
/// read/write behavior.
#[test]
fn nestest() -> wadatsumi::Result<()> {
    let mut system = System::new();
    system.open("tests/nestest/nestest.nes")?;

    // Skip the reset vector and jump straight to the automation entry point.
    // The normal reset vector initialises the PPU, which we have not
    // implemented yet; $C000 bypasses all of that.
    system.cpu.state.pc = 0xc000;

    // Run for exactly the number of cycles the official-opcode section
    // requires. The ROM halts itself via an infinite loop at this point, so
    // running additional cycles would be harmless, but the fixed budget makes
    // the test deterministic and prevents us from accidentally executing the
    // unofficial-opcode section.
    for _ in 0..26_554 {
        system.tick()?;
    }

    // A non-zero value here means at least one opcode produced the wrong
    // result; the value itself is a lookup key in nestest.txt.
    assert_eq!(system.bus.read(0x0002), 0x00, "nestest result code");

    // Non-zero only when $0002 is also non-zero; narrows the failure to a
    // specific sub-test within the failing group.
    assert_eq!(system.bus.read(0x0003), 0x00, "nestest sub-test code");

    Ok(())
}
