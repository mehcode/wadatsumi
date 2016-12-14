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

#[macro_use]
mod om;

mod op;
mod operation;

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

    let window = WindowBuilder::new(&video, "Wadatsumi", 160 * 4, 144 * 4).build().unwrap();

    let mut m = machine::Machine::new();

    // let filename = "/Users/mehcode/Workspace/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb";
    let filename = "/Users/mehcode/Documents/Games/Dr. Mario.gb";
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
        // Run: Machine
        m.run();

        // Render: Clear the window
        renderer.set_draw_color(Color::RGB(255, 255, 255));
        renderer.clear();

        // Render: Present
        renderer.present();

        // Poll events
        if let Some(evt) = events.poll_event() {
            match evt {
                Event::Quit { .. } => {
                    // Quit the program
                    is_running = false;
                }

                Event::KeyDown { scancode, repeat, .. } => {
                    if !repeat {
                        if let Some(scancode) = scancode {
                            info!("event: key down: {}", scancode);
                        }
                    }
                }

                Event::KeyUp { scancode, repeat, .. } => {
                    if !repeat {
                        if let Some(scancode) = scancode {
                            info!("event: key up: {}", scancode);
                        }
                    }
                }

                _ => {
                    // Unhandled event
                }
            }
        }
    }
}
