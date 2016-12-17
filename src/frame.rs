pub struct Frame<'a> {
    // Pixel data
    pub data: &'a [u8],

    // Pixel pitch
    pub pitch: usize,

    // Width (in pixels)
    pub width: usize,

    // Height (in pixels)
    pub height: usize,
}
