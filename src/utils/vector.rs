use crate::utils::yaml::{parse_float, parse_float_array, FromYaml, YamlPropertyError};
use crate::utils::{random_float, random_range};
use std::f64::consts::{FRAC_1_PI, FRAC_2_PI, PI};
use std::fmt::{Display, Formatter};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};
use yaml_rust::Yaml;

#[derive(Debug, Clone, Copy)]
/// A three element vector with lots of operators and common functions.
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    /// Constructs a new vector with given x, y, and z values.
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }

    /// Constructs a new vector from a given list of values.
    pub fn from_values_list(values: Vec<f64>) -> Vector {
        Vector {
            x: values[0],
            y: values[1],
            z: values[2],
        }
    }

    /// Returns a zero vector.
    pub fn zero() -> Vector {
        Vector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Constructs a vector with random components in a given range.
    pub fn random(min: f64, max: f64) -> Vector {
        Vector::new(
            random_range(min, max),
            random_range(min, max),
            random_range(min, max),
        )
    }

    /// Returns a random vector in a unit sphere.
    pub fn random_in_unit_sphere() -> Vector {
        // Uses rejection sampling to find a valid random vector
        loop {
            // Creates a random vector within a square
            let v = Vector::random(-1.0, 1.0);

            // Returns the vector if it lies within the unit sphere
            if v.len_sqr() <= 1.0 {
                return v;
            }
        }
    }

    /// Returns a random vector in a hemisphere around the given normal.
    pub fn random_in_hemisphere(normal: &Vector) -> Vector {
        // Gets a random vector in a unit sphere
        let v = Vector::random_in_unit_sphere();

        // Flips the vector if it faces away from the normal
        if Vector::dot(&v, normal) >= 0.0 {
            v
        } else {
            -v
        }
    }

    /// Returns a cosine weighted random vector in a hemisphere around the given normal for diffuse sampling.
    /// Based on Lafortune's importance sampling method from https://www.researchgate.net/publication/2361953_Using_the_Modied_Phong_Reflectance_Model_for_Physically_Based_Rendering
    pub fn sample_diffuse_vector(normal: &Vector) -> Vector {
        // Calculates two required random variables
        let eta_1 = random_float();
        let eta_2 = random_float();

        // Calculates spherical coordinates of output vector
        let theta = eta_1.sqrt().acos();
        let phi = eta_2 * PI * 2.0;

        // Gets x, y, and z components
        let x = theta.sin() * phi.cos();
        let y = theta.sin() * phi.sin();
        let z = theta.cos();

        // Creates an orthonormal basis for the given normal vector
        let tangent = if normal.x.abs() > 0.5 {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(1.0, 0.0, 0.0)
        };
        let bitangent = Vector::cross(normal, &tangent).unit();
        let tangent = Vector::cross(&bitangent, normal);

        // Returns the final sampled direction
        (tangent * x + bitangent * y + *normal * z).unit()
    }

    /// Returns a random vector in a hemisphere around the given ideal vector,
    /// importance sampled for glossy specular reflection. Based on Lafortune's importance
    /// sampling method from https://www.researchgate.net/publication/2361953_Using_the_Modied_Phong_Reflectance_Model_for_Physically_Based_Rendering
    pub fn sample_specular_vector(ideal_dir: &Vector, cos_i: f64, alpha: f64) -> Vector {
        // Returns ideal direction directly for alpha values less than 1
        if alpha < 1.0 {
            return ideal_dir.clone();
        }

        // Calculates a limit based on the angle of incidence to ensure rays aren't reflected into
        // the surface
        let limit = 1.0 - cos_i.acos() * FRAC_2_PI;

        // Calculates two required random variables
        let eta_1 = random_float();
        let eta_2 = random_float();

        // Calculates spherical coordinates of output vector
        let theta = eta_1.powf(1.0 / (alpha + 1.0)).acos() * limit;
        let phi = eta_2 * PI * 2.0;

        // Gets x, y, and z components
        let x = theta.sin() * phi.cos();
        let y = theta.sin() * phi.sin();
        let z = theta.cos();

        // Creates an orthonormal basis for the given ideal reflection vector
        let tangent = if ideal_dir.x.abs() > 0.5 {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(1.0, 0.0, 0.0)
        };
        let bitangent = Vector::cross(ideal_dir, &tangent).unit();
        let tangent = Vector::cross(&bitangent, ideal_dir);

        // Returns the final sampled direction
        (tangent * x + bitangent * y + *ideal_dir * z).unit()
    }

    /// Returns a random unit vector.
    pub fn random_unit_vector() -> Vector {
        Self::random_in_unit_sphere().unit()
    }

    /// Returns the squared length of the vector.
    pub fn len_sqr(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the length of the vector.
    pub fn length(&self) -> f64 {
        f64::sqrt(self.len_sqr())
    }

    /// Normalises the vector.
    pub fn normalise(&mut self) {
        let length_recip = 1.0 / self.length();
        self.x = self.x * length_recip;
        self.y = self.y * length_recip;
        self.z = self.z * length_recip;
    }

    /// Returns the unit (normalised) vector.
    pub fn unit(self) -> Vector {
        (1. / self.length()) * self
    }

    /// Computes the dot product of two given vectors.
    pub fn dot(u: &Vector, v: &Vector) -> f64 {
        u.x * v.x + u.y * v.y + u.z * v.z
    }

    /// Computes the cross product of two given vectors.
    pub fn cross(u: &Vector, v: &Vector) -> Vector {
        Vector::new(
            u.y * v.z - u.z * v.y,
            u.z * v.x - u.x * v.z,
            u.x * v.y - u.y * v.x,
        )
    }

    /// Computes the reflection of a vector in the normal of a surface.
    pub fn reflection(normal: &Vector, v: &Vector) -> Vector {
        let d = 2. * Vector::dot(normal, v);

        Vector::new(v.x - d * normal.x, v.y - d * normal.y, v.z - d * normal.z)
    }
}

/// Implements vector addition.
impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

/// Implements vector addition with the '+=' operator.
impl AddAssign for Vector {
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/// Implements negation of vectors using the unary '-'.
impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y, -self.z)
    }
}

/// Implements vector subtraction {
impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// Implements vector subtraction with the '-=' operator.
impl SubAssign for Vector {
    fn sub_assign(&mut self, other: Vector) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

/// Implements element-wise multiplication of two vectors.
impl Mul for Vector {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        Vector::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

/// Implements multiplication of a vector with a scalar value
impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, value: f64) -> Vector {
        Vector::new(self.x * value, self.y * value, self.z * value)
    }
}

/// Implements multiplication of a scalar value with a vector
impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, vector: Vector) -> Vector {
        Vector::new(self * vector.x, self * vector.y, self * vector.z)
    }
}

/// Implements multiplication of a vector by a scalar value with the '*=' operator.
impl MulAssign<f64> for Vector {
    fn mul_assign(&mut self, value: f64) {
        self.x *= value;
        self.y *= value;
        self.z *= value;
    }
}

/// Implements division of a vector by a scalar value.
impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, value: f64) -> Vector {
        Vector::new(self.x / value, self.y / value, self.z / value)
    }
}

/// Implements division of a vector by a scalar value with the '/=' operator.
impl DivAssign<f64> for Vector {
    fn div_assign(&mut self, value: f64) {
        self.x /= value;
        self.y /= value;
        self.z /= value;
    }
}

/// Implements indexing a vector to access its elements.
impl Index<u8> for Vector {
    type Output = f64;

    fn index(&self, i: u8) -> &f64 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index {} out of range for Vector struct.", i),
        }
    }
}

/// Implements indexing a vector to modify elements.
impl IndexMut<u8> for Vector {
    fn index_mut(&mut self, i: u8) -> &mut f64 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index {} out of range for Vector struct.", i),
        }
    }
}

/// Provides vector formatting and printing.
impl Display for Vector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{} y:{} z:{}", self.x, self.y, self.z)
    }
}

/// Implements loading vectors from a YAML file.
impl FromYaml for Vector {
    fn from_yaml(yaml: &Yaml) -> Result<Vector, YamlPropertyError> {
        // Attempts to construct the vector by treating the `Yaml` instance as either an array or hashmap
        if let Yaml::Array(array) = yaml {
            // Checks that the given array is exactly 3 elements long
            if array.len() != 3 {
                return Err(YamlPropertyError::invalid("array"));
            }

            // Parses the array elements as floats and uses them to construct a vector
            let props = parse_float_array(array)?;
            Ok(Vector::from_values_list(props))
        } else {
            // Parses individual properties from the `Yaml` instance
            let x = parse_float(yaml, "x")?;
            let y = parse_float(yaml, "y")?;
            let z = parse_float(yaml, "z")?;

            // Returns the new vector instance
            Ok(Vector::new(x, y, z))
        }
    }
}
