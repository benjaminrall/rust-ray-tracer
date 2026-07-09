#[derive(Clone)]
/// Struct to represent a pixel in a `FrameBuffer`.
pub struct Pixel {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Pixel {
    /// Creates a new `Pixel` instance with the given RGB values.
    pub fn new(red: f64, green: f64, blue: f64) -> Pixel {
        Pixel { red, green, blue }
    }
}
