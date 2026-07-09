//! Light sources to illuminate scenes.

mod directional_quad_light;
mod light;
mod point_light;
mod quad_light;
mod sphere_light;

pub use directional_quad_light::DirectionalQuadLight;
pub use light::{Light, LightTrait};
pub use point_light::PointLight;
pub use quad_light::QuadLight;
pub use sphere_light::SphereLight;
