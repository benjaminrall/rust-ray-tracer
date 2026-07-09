#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
/// Enum to represent which side of a splitting plane an object should be placed during KDTree construction.
pub enum ObjectSide {
    Left,
    Right,
}
