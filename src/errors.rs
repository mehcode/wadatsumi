use std::io;
use log;
use sdl2;

error_chain! {
    foreign_links {
        Io(io::Error);
        LogSetLogger(log::SetLoggerError);
        Sdl2VideoWindowBuild(sdl2::video::WindowBuildError);
        Sdl2IntegerOrSdl(sdl2::IntegerOrSdlError);
    }
}
