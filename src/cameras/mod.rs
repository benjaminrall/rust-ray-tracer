//! Camera models for ray generation.

mod camera;
mod full_camera;
mod realistic_camera;
mod simple_camera;

pub use camera::{Camera, CameraTrait};
pub use full_camera::FullCamera;
pub use realistic_camera::RealisticCamera;
pub use simple_camera::SimpleCamera;
