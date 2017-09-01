extern crate sdl2;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod errors;
mod cartridge;

use std::env;
use std::fs::File;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use errors::*;

quick_main!(|| -> Result<()> {
    pretty_env_logger::init()?;

    let argv = env::args().collect::<Vec<_>>();
    let filename = &argv[1];

    let cart = cartridge::Cartridge::from_reader(File::open(filename)?)?;

    //let sdl_context = sdl2::init()?;
    //let video_subsystem = sdl_context.video()?;
    //let window = video_subsystem.window("Wadatsumi", 800, 600)
    //    .position_centered()
    //    .opengl()
    //    .build()?;
    //
    //let mut canvas = window.into_canvas()
    //    .accelerated()
    //    .present_vsync()
    //    .build()?;
    //
    //canvas.set_draw_color(Color::RGB(40, 40, 40));
    //canvas.clear();
    //canvas.present();
    //
    //let mut event_pump = sdl_context.event_pump()?;
    //
    //'outer: loop {
    //    for event in event_pump.poll_iter() {
    //        match event {
    //            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
    //                break 'outer;
    //            },
    //
    //            _ => { }
    //        }
    //    }
    //
    //    canvas.clear();
    //    canvas.present();
    //}
    //
    Ok(())
});
