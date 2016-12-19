use sdl2::keyboard;
use std::io;

pub trait Machine {
    fn open(&mut self, filename: &str) -> io::Result<()>;

    fn reset(&mut self);

    fn run(&mut self);

    fn on_key_down(&mut self, scancode: keyboard::Scancode);

    fn on_key_up(&mut self, scancode: keyboard::Scancode);

    fn set_on_video_refresh(&mut self, callback: Box<FnMut(::frame::Frame) -> ()>);
    fn set_on_sound_refresh(&mut self, callback: Box<FnMut(&[i16]) -> ()>);

    /// Get (initial) display width
    fn get_width(&self) -> u32;

    /// Get (initial) display height
    fn get_height(&self) -> u32;
}
