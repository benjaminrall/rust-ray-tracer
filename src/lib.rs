//! Root module for the ray tracer library.
//!
//! Exports core rendering modules and defines global constants used throughout the software.

// Module definition
pub mod cameras;
pub mod core;
pub mod drawing;
pub mod lights;
pub mod materials;
pub mod objects;
pub mod textures;
pub mod utils;

// Constants
pub const EPSILON: f64 = 1e-8; // Value for numerical stability
pub const WIDTH: i64 = 512; // Default width of the image
pub const HEIGHT: i64 = 512; // Default height of the image
pub const MAX_PHOTON_TRACE_DEPTH: usize = 128; // Maximum depth that photons can travel through the scene
pub const MAX_INDIRECT_DEPTH: usize = 128; // Maximum depth that rays for indirect calculations can travel through the scene
pub const INDIRECT_SAMPLES: usize = 8; // Number of rays sample for indirect lighting
pub const DIRECT_SAMPLES: usize = 8; // Number of rays to sample for direct lighting
pub const SPECULAR_SAMPLES: usize = 4; // Number of rays to sample for glossy specular reflections/refractions
pub const MAX_RECURSE: i32 = 8; // Maximum depth that a ray can recurse
pub const CONE_FILTER_K: f64 = 1.0; // Coefficient for the cone filter for radiance estimates
pub const AIR_IOR: f64 = 1.; // Index of refraction of air
