#![feature(concat_idents)]

extern crate sdl2;
extern crate strfmt;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;
extern crate env_logger;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::video::WindowBuilder;
use sdl2::render::RendererBuilder;

mod operation;

#[macro_use]
mod om;

mod op;
mod cpu;
mod machine;
mod bus;
mod cart;

fn main() {
    env_logger::init().unwrap();

    let c = sdl2::init().unwrap();
    let mut events = c.event_pump().unwrap();
    let video = c.video().unwrap();

    let mut is_running = true;

    let mut window = WindowBuilder::new(&video, "Wadatsumi", 160 * 4, 144 * 4).build().unwrap();

    let mut m = machine::Machine::new();

    // let filename = "/Users/mehcode/Workspace/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb";
    let filename = "/Users/mehcode/Documents/Games/Tetris.gb";
    m.open(filename).unwrap();

    m.reset();

    // Update title on window
    // TODO(wadatsumi): relativize the filename
    // window.set_title(format!("Wadatsumi — {}",
    //                       if m.cart.title.is_empty() {
    //                           &filename
    //                       } else {
    //                           m.cart.title.as_str()
    //                       })
    //        .as_str())
    //    .unwrap();

    let mut renderer = RendererBuilder::new(window).accelerated().build().unwrap();

    while is_running {
        m.run();

        // println!("{:>6}: {:<40} PC: 0x{:04X} AF: 0x{:04X} BC: 0x{:04X} DE: 0x{:04X} HL: \
        //          0x{:04X} SP: 0x{:04X}",
        //         cycles,
        //         strfmt::strfmt_map(op.disassembly, &|mut fmt: strfmt::Formatter| fmt.i64(1))
        //             .unwrap(),
        //         10,
        //         14,
        //         50,
        //         120,
        //         30,
        //         63);

        // Render: Clear the window
        renderer.set_draw_color(Color::RGB(255, 255, 255));
        renderer.clear();

        // Render: present
        renderer.present();

        // Poll events
        if let Some(evt) = events.poll_event() {
            match evt {
                Event::Quit { .. } => {
                    // Quit the program
                    is_running = false;
                }

                _ => {
                    // Unhandled event
                }
            }
        }
    }
}
