#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
/// Enum to represent the different ways in which a photon can be scattered by a material.
pub enum ScatterType {
    Diffuse,
    Specular,
    Volume,
}
