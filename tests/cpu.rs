#![feature(conservative_impl_trait)]

extern crate flate2;
extern crate reqwest;
extern crate tar;
extern crate wadatsumi;

use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;
use std::sync::{Once, ONCE_INIT};
use wadatsumi::{Cpu, Bus};

static START: Once = ONCE_INIT;

// Download the test roms to .cache/ (only if we don't have them already)
// FIXME: Duplicated code with benches/
fn before() {
    const URI_GB_TEST_ROMS: &str = "https://github.com/mehcode/gb-test-roms/archive/master.tar.gz";

    START.call_once(|| {
        if !Path::new(".cache/gb-test-roms-master").exists() {
            let response = reqwest::get(URI_GB_TEST_ROMS).unwrap();
            let mut archive = Archive::new(GzDecoder::new(response).unwrap());

            fs::create_dir_all(".cache").unwrap();

            archive.unpack(".cache").unwrap();
        }
    });
}

// Blargg's cpu tests output to SD. We capture that here.
#[derive(Default)]
struct SerialDataCapture(Rc<RefCell<String>>);

impl Bus for SerialDataCapture {
    fn contains(&self, address: u16) -> bool {
        0xFF01 == address
    }

    fn write8(&mut self, _: u16, value: u8) {
        self.0.borrow_mut().push(value as char);
    }
}

// Setup a barebones emulator to run a test _just_ the CPU with
fn before_each(filename: &str) -> (Cpu, impl Bus, Rc<RefCell<String>>) {
    let file = fs::File::open(&format!(".cache/gb-test-roms-master/cpu_instrs/individual/{}.gb", filename)).unwrap();

    let cpu = Cpu::new();

    let cartridge = wadatsumi::Cartridge::from_reader(file).unwrap();
    let work_ram = wadatsumi::WorkRam::new();
    let high_ram = wadatsumi::HighRam::new();

    let output = Rc::new(RefCell::new(String::new()));
    let serial_data_capture = SerialDataCapture(output.clone());

    (cpu, (cartridge, (work_ram, (high_ram, serial_data_capture))), output)
}

macro_rules! make_test {
    ($id:ident : $filename:expr) => {
        #[test]
        fn $id() {
            before();
            let (mut cpu, mut bus, output) = before_each($filename);

            // FIXME: This should run until HALT, inf. loop, etc. detected
            for _ in 0..(25_000_000) {
                cpu.run_next(&mut bus);
            }

            // Compare output and ensure we have a success indication
            assert!(output.borrow().contains("Passed"));

            // Print output for debugging
            println!("{}", output.borrow());
        }
    }
}

make_test!(cpu_instrs_01: "01-special");
make_test!(cpu_instrs_02: "02-interrupts");
make_test!(cpu_instrs_03: "03-op sp,hl");
make_test!(cpu_instrs_04: "04-op r,imm");
make_test!(cpu_instrs_05: "05-op rp");
make_test!(cpu_instrs_06: "06-ld r,r");
make_test!(cpu_instrs_07: "07-jr,jp,call,ret,rst");
make_test!(cpu_instrs_08: "08-misc instrs");
make_test!(cpu_instrs_09: "09-op r,r");
make_test!(cpu_instrs_10: "10-bit ops");
make_test!(cpu_instrs_11: "11-op a,(hl)");
