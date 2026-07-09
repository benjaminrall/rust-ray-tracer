#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
/// Enum representing filter types used for estimating radiance from a photon map.
pub enum FilterType {
    Disk,
    Sphere,
    Cone,
}
