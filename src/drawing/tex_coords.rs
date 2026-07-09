use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
/// Struct to represent texture coordinates.
pub struct TexCoords {
    pub u: f64,
    pub v: f64,
    pub w: f64,
}

impl TexCoords {
    /// Creates a new texture coordinates object.
    pub fn new(u: f64, v: f64, w: f64) -> TexCoords {
        TexCoords { u, v, w }
    }
}

/// Implements addition of texture coordinates.
impl Add for TexCoords {
    type Output = TexCoords;

    fn add(self, other: TexCoords) -> TexCoords {
        TexCoords::new(self.u + other.u, self.v + other.v, self.w + other.w)
    }
}

/// Implements multiplication of texture coordinates by a scalar value.
impl Mul<f64> for TexCoords {
    type Output = TexCoords;

    fn mul(self, other: f64) -> TexCoords {
        TexCoords::new(self.u * other, self.v * other, self.w * other)
    }
}

/// Implements multiplication of a scalar value by texture coordinates.
impl Mul<TexCoords> for f64 {
    type Output = TexCoords;

    fn mul(self, other: TexCoords) -> TexCoords {
        TexCoords::new(other.u * self, other.v * self, other.w * self)
    }
}
