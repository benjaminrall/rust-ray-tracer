use crate::utils::yaml::{parse_float, parse_float_array, FromYaml, YamlPropertyError};
use crate::utils::Vector;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};
use yaml_rust::Yaml;

#[derive(Debug)]
/// A four element vector with lots of operators and common functions.
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vertex {
    /// Constructs a new vertex with given x, y, and z values.
    pub fn new(x: f64, y: f64, z: f64) -> Vertex {
        Vertex { x, y, z, w: 1. }
    }

    /// Constructs a new vertex from a given list of values.
    pub fn from_values_list(values: Vec<f64>) -> Vertex {
        Vertex {
            x: values[0],
            y: values[1],
            z: values[2],
            w: 1.,
        }
    }

    /// Constructs a new vertex with given x, y, z, and w values.
    pub fn with_w(x: f64, y: f64, z: f64, w: f64) -> Vertex {
        Vertex { x, y, z, w }
    }

    /// Constructs a new vertex from a given vector.
    pub fn from_vector(v: &Vector) -> Vertex {
        Vertex::new(v.x, v.y, v.z)
    }

    /// Returns a zero vertex.
    pub fn zero() -> Vertex {
        Vertex {
            x: 0.,
            y: 0.,
            z: 0.,
            w: 1.,
        }
    }

    /// Converts a vertex to a vector.
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }
}

/// Allows vertices to be cloned directly.
impl Clone for Vertex {
    fn clone(&self) -> Self {
        Vertex::with_w(self.x, self.y, self.z, self.w)
    }
}
impl Copy for Vertex {}

/// Implements addition of vectors to vertices.
impl Add<Vector> for Vertex {
    type Output = Vertex;

    fn add(self, other: Vector) -> Vertex {
        Vertex::with_w(self.x + other.x, self.y + other.y, self.z + other.z, self.w)
    }
}

/// Implements addition with the '+=' operator.
impl AddAssign<Vector> for Vertex {
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/// Implements negation of vertices using the unary '-'.
impl Neg for Vertex {
    type Output = Vertex;

    fn neg(self) -> Vertex {
        Vertex::with_w(-self.x, -self.y, -self.z, -self.w)
    }
}

/// Implements subtraction of vectors from vertices.
impl Sub<Vector> for Vertex {
    type Output = Vertex;

    fn sub(self, other: Vector) -> Vertex {
        Vertex::with_w(self.x - other.x, self.y - other.y, self.z - other.z, self.w)
    }
}

/// Implements subtraction of vertices from vectors.
impl Sub<Vertex> for Vector {
    type Output = Vector;

    fn sub(self, other: Vertex) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// Implements subtraction of vectors from vertices.
impl Sub for Vertex {
    type Output = Vector;

    fn sub(self, other: Vertex) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// Implements subtraction with the '-=' operator.
impl SubAssign<Vector> for Vertex {
    fn sub_assign(&mut self, other: Vector) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

/// Implements multiplication of a vertex with a scalar value.
impl Mul<f64> for Vertex {
    type Output = Vertex;

    fn mul(self, value: f64) -> Vertex {
        Vertex::new(self.x * value, self.y * value, self.z * value)
    }
}

/// Implements multiplication of a scalar value with a vertex.
impl Mul<Vertex> for f64 {
    type Output = Vertex;

    fn mul(self, vertex: Vertex) -> Vertex {
        Vertex::new(self * vertex.x, self * vertex.y, self * vertex.z)
    }
}

/// Implements multiplication of a vertex with a scalar value using the '*=' operator.
impl MulAssign<f64> for Vertex {
    fn mul_assign(&mut self, value: f64) {
        self.x *= value;
        self.y *= value;
        self.z *= value;
    }
}

/// Implements division of a vertex by a scalar value with the '/=' operator.
impl DivAssign<f64> for Vertex {
    fn div_assign(&mut self, value: f64) {
        self.x /= value;
        self.y /= value;
        self.z /= value;
        self.w /= value;
    }
}

/// Implements indexing a vertex to access its elements.
impl Index<u8> for Vertex {
    type Output = f64;

    fn index(&self, i: u8) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Index {} out of range for Vertex struct.", i),
        }
    }
}

/// Implements indexing a vertex to modify elements.
impl IndexMut<u8> for Vertex {
    fn index_mut(&mut self, i: u8) -> &mut f64 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Index {} out of range for Vertex struct.", i),
        }
    }
}

/// Provides vertex formatting and printing.
impl Display for Vertex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{} y:{} z:{} w:{}", self.x, self.y, self.z, self.w)
    }
}

/// Implements loading vertices from a YAML file.
impl FromYaml for Vertex {
    fn from_yaml(yaml: &Yaml) -> Result<Vertex, YamlPropertyError> {
        // Attempts to construct the vertex by treating the `Yaml` instance as either an array or hashmap
        if let Yaml::Array(array) = yaml {
            // Checks that the given array is exactly 3 elements long
            if array.len() != 3 {
                return Err(YamlPropertyError::invalid("array"));
            }

            // Parses the array elements as floats and uses them to construct a vertex
            let props = parse_float_array(array)?;
            Ok(Vertex::from_values_list(props))
        } else {
            // Parses individual properties from the `Yaml` instance
            let x = parse_float(yaml, "x")?;
            let y = parse_float(yaml, "y")?;
            let z = parse_float(yaml, "z")?;

            // Returns the new vertex instance
            Ok(Vertex::new(x, y, z))
        }
    }
}
