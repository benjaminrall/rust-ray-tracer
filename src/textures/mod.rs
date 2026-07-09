//! Procedural and image-based texture mapping.

mod checker_texture;
mod colour_texture;
mod image_texture;
mod marble_texture;
mod noise_texture;
mod texture;
mod y_gradient_texture;

pub use checker_texture::CheckerTexture;
pub use colour_texture::ColourTexture;
pub use image_texture::ImageTexture;
pub use noise_texture::NoiseTexture;
pub use texture::{Texture, TextureTrait};
