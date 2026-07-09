//! Data structures related to colour manipulation and writing to image buffers.

mod colour;
mod frame_buffer;
mod pixel;
mod tex_coords;

pub use colour::{linear_to_srgb, srgb_to_linear, Colour};
pub use frame_buffer::FrameBuffer;
pub use pixel::Pixel;
pub use tex_coords::TexCoords;
