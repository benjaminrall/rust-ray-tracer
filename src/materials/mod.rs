//! BRDFs and material properties which dictate how rays scatter.

mod emissive_material;
mod emissive_volume_material;
mod global_material;
mod lambertian_material;
mod material;
mod phong_material;
mod volume_material;

pub use emissive_material::EmissiveMaterial;
pub use emissive_volume_material::EmissiveVolumeMaterial;
pub use global_material::GlobalMaterial;
pub use lambertian_material::LambertianMaterial;
pub use material::{Material, MaterialTrait};
pub use phong_material::PhongMaterial;
pub use volume_material::VolumeMaterial;
