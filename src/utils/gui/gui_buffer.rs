use crate::drawing::Colour;

/// Wrapper for the Ray Tracer GUI's image data RGB buffer.
pub struct GUIBuffer {
    width: usize,
    height: usize,
    image_data: Vec<u8>,
}

impl GUIBuffer {
    /// Creates a new empty GUI buffer with a given width and height.
    pub fn new(width: usize, height: usize) -> GUIBuffer {
        let image_data = vec![0; width * height * 3];
        GUIBuffer {
            width,
            height,
            image_data,
        }
    }

    /// Returns a reference to the internal image data vector.
    pub fn get_ref(&self) -> &[u8] {
        &self.image_data
    }

    /// Sets the pixel at the given coordinates to the given colour.
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let idx = (y * self.width + x) * 3;
        self.image_data[idx] = r;
        self.image_data[idx + 1] = g;
        self.image_data[idx + 2] = b;
    }
}
