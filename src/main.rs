#[macro_use]
extern crate clap;

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

use clap::{Arg, App};

mod mode;
mod bits;
mod frame;
mod sound;
mod machine;
mod gb;  // Gameboy, Gameboy Color (*), Super Gameboy (*)

use machine::Machine;

fn main() {
    // Log: Initialize (level set from environment variables)
    // TODO: Switch to use: https://github.com/slog-rs/slog
    env_logger::init().unwrap();

    // Configure and gather matches from command line interface
    let matches = App::new("Wadatsumi")
        .version(crate_version!())
        .arg(Arg::with_name("scale")
            .short("s")
            .long("scale")
            .takes_value(true)
            .default_value("2")
            .help("The scale factor to apply to the source display"))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .takes_value(true)
            .help("The device (and variation) to emulate")
            // TODO: This should be generated
            .possible_values(&["gb", "gb:dmg0", "gb:dmg", "gb:mgb", "gb:cgb", "gb:agb", "gb:sgb", "gb:sgb1", "gb:sgb2", "cgb",
                               "cgb:cgb", "cgb:agb", "sgb", "sgb:1", "sgb:2"]))
        .arg(Arg::with_name("rom").required(true).help("The ROM to use"))
        .get_matches();

    let scale = matches.value_of("scale").unwrap();
    let scale = scale.parse::<u32>().unwrap();

    let rom_filename = matches.value_of("rom").unwrap();

    let mode: Option<::mode::Mode> = match matches.value_of("mode") {
        Some(mode_str) => ::mode::Mode::from_str(mode_str),
        _ => None,
    };

    let c = sdl2::init().unwrap();
    let mut events = c.event_pump().unwrap();
    let video = c.video().unwrap();

    let mut is_running = true;

    let mut m: Box<Machine> = Box::new(gb::machine::Machine::new(mode));
    m.open(rom_filename).unwrap();

    // Create audio queue
    let audio = c.audio().unwrap();
    let audio_queue = audio.open_queue::<i16>(None,
                           &sdl2::audio::AudioSpecDesired {
                               samples: Some(::sound::BUFFER_SIZE as u16),
                               channels: Some(2),
                               freq: Some(::sound::SAMPLE_RATE as i32),
                           })
        .unwrap();

    audio_queue.resume();

    m.set_on_sound_refresh(Box::new(move |buffer| {
        // Sync to audio queue
        while audio_queue.size() > (::sound::BUFFER_SIZE as u32) * 2 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        audio_queue.queue(buffer);
    }));

    // Create window
    let width = m.get_width();
    let height = m.get_height();
    let window =
        WindowBuilder::new(&video, "Wadatsumi", width * scale, height * scale).build().unwrap();

    // Create 2D renderer
    // TODO: Do not use present_vsync and instead limit frame rate manually
    let mut renderer = RendererBuilder::new(window).accelerated().present_vsync().build().unwrap();

    // Initially clear the renderer
    renderer.set_draw_color(Color::RGB(255, 255, 255));
    renderer.clear();
    renderer.present();

    // Create texture for framebuffer
    let mut texture =
        renderer.create_texture_streaming(sdl2::pixels::PixelFormatEnum::ARGB8888, width, height)
            .unwrap();

    m.set_on_video_refresh(Box::new(move |frame| {
        // Render: Update texture and flip
        texture.update(None, frame.data, frame.pitch).unwrap();
        renderer.copy(&texture, None, None).unwrap();

        // Render: Present
        renderer.present();
    }));

    m.reset();

    while is_running {
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
                            m.on_key_down(scancode);
                        }
                    }
                }

                Event::KeyUp { scancode, repeat, .. } => {
                    if !repeat {
                        if let Some(scancode) = scancode {
                            m.on_key_up(scancode);
                        }
                    }
                }

                _ => {
                    // Unhandled event
                }
            }
        }

        // Run: Machine (for 5000 cycles)
        for _ in 1..5000 {
            m.run();
        }
    }
}
