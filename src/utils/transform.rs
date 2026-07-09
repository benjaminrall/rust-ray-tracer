use crate::utils::yaml::{
    parse_float, parse_float_array, parse_struct, parse_vec, ExtendYamlResult, FromYaml,
    YamlPropertyError,
};
use crate::utils::{Vector, Vertex};
use std::ops::{Index, Mul};
use yaml_rust::Yaml;

#[derive(Debug, Clone, Copy)]
/// Struct to store, manipulate, and apply transformations.
pub struct Transform {
    matrix: [[f64; 4]; 4],          // Transformation matrix
    inverse: Option<[[f64; 4]; 4]>, // Inverse of the transformation matrix, if one exists

    inverse_calculated: bool, // Whether the inverse has been pre-computed yet
}

impl Transform {
    /// Creates a new Transform object from given unpacked matrix values.
    pub fn new(
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
        o: f64,
        p: f64,
    ) -> Transform {
        let matrix = [[a, b, c, d], [e, f, g, h], [i, j, k, l], [m, n, o, p]];
        Transform {
            matrix,
            inverse: None,
            inverse_calculated: false,
        }
    }

    /// Creates a new Transform object from given unpacked matrix values and precomputes the inverse.
    pub fn with_inverse(
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        k: f64,
        l: f64,
        m: f64,
        n: f64,
        o: f64,
        p: f64,
    ) -> Transform {
        let mut t = Transform::new(a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p);
        t.calculate_inverse();
        t
    }

    /// Creates a new Transform object from a given matrix array.
    pub fn from_array(matrix: [[f64; 4]; 4]) -> Transform {
        Transform {
            matrix,
            inverse: None,
            inverse_calculated: false,
        }
    }

    /// Creates a new Transform object from a given matrix array and precomputes the inverse.
    pub fn from_array_with_inverse(matrix: [[f64; 4]; 4]) -> Transform {
        let mut t = Transform::from_array(matrix);
        t.calculate_inverse();
        t
    }

    /// Returns the identity transform.
    pub fn identity() -> Transform {
        let matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        Transform {
            matrix,
            inverse: Some(matrix),
            inverse_calculated: true,
        }
    }

    /// Returns a zero transform.
    pub fn zero() -> Transform {
        Transform {
            matrix: [[0.0; 4]; 4],
            inverse: None,
            inverse_calculated: false,
        }
    }

    /// Returns a translation matrix.
    pub fn translate(tx: f64, ty: f64, tz: f64) -> Transform {
        Transform::from_array([
            [1.0, 0.0, 0.0, tx],
            [0.0, 1.0, 0.0, ty],
            [0.0, 0.0, 1.0, tz],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a rotation matrix for theta degrees anti-clockwise around the x-axis.
    pub fn rotate_x(theta: f64) -> Transform {
        let c = theta.to_radians().cos();
        let s = theta.to_radians().sin();
        Transform::from_array([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, -s, 0.0],
            [0.0, s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a rotation matrix for theta degrees anti-clockwise around the y-axis.
    pub fn rotate_y(theta: f64) -> Transform {
        let c = theta.to_radians().cos();
        let s = theta.to_radians().sin();
        Transform::from_array([
            [c, 0.0, s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-s, 0.0, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a rotation matrix for theta degrees anti-clockwise around the z-axis.
    pub fn rotate_z(theta: f64) -> Transform {
        let c = theta.to_radians().cos();
        let s = theta.to_radians().sin();
        Transform::from_array([
            [c, -s, 0.0, 0.0],
            [s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a matrix for scaling uniformly along all three axes by the given scale factor.
    pub fn scale(sf: f64) -> Transform {
        Transform::from_array([
            [sf, 0.0, 0.0, 0.0],
            [0.0, sf, 0.0, 0.0],
            [0.0, 0.0, sf, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a matrix for stretching by a given scale factor along the x-axis.
    pub fn stretch_x(sf: f64) -> Transform {
        Transform::from_array([
            [sf, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a matrix for stretching by a given scale factor along the y-axis.
    pub fn stretch_y(sf: f64) -> Transform {
        Transform::from_array([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, sf, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Returns a matrix for stretching by a given scale factor along the z-axis.
    pub fn stretch_z(sf: f64) -> Transform {
        Transform::from_array([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, sf, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Applies the transform to a vertex in-place.
    pub fn apply_vertex(&self, u: &mut Vertex) {
        // Initialises the output vertex
        let mut v = Vertex::zero();

        // Performs matrix multiplication
        for i in 0..4 {
            v[i as u8] = self.matrix[i][0] * u.x
                + self.matrix[i][1] * u.y
                + self.matrix[i][2] * u.z
                + self.matrix[i][3] * u.w;
        }

        // Updates original vertex with the transformed value
        *u = v;
    }

    /// Applies the transposed transform to a vertex in-place.
    pub fn apply_transpose_vertex(&self, u: &mut Vertex) {
        // Initialises the output vertex
        let mut v = Vertex::zero();

        // Performs matrix multiplication
        for i in 0..4 {
            v[i as u8] = self.matrix[0][i] * u.x
                + self.matrix[1][i] * u.y
                + self.matrix[2][i] * u.z
                + self.matrix[3][i] * u.w;
        }

        // Updates original vertex with the transformed value
        *u = v;
    }

    /// Applies the inverse transform to a vertex in-place.
    pub fn apply_inverse_vertex(&self, u: &mut Vertex) {
        // Checks that the inverse exists and gets a reference to it
        self.check_inverse();
        let inverse = self.inverse.as_ref().unwrap();

        // Initialises the output vertex
        let mut v = Vertex::zero();

        // Performs matrix multiplication
        for i in 0..4 {
            v[i as u8] = inverse[i][0] * u.x
                + inverse[i][1] * u.y
                + inverse[i][2] * u.z
                + inverse[i][3] * u.w;
        }

        // Updates original vertex with the transformed value
        *u = v;
    }

    /// Applies the transposed inverse transform to a vertex in-place.
    pub fn apply_transpose_inverse_vertex(&self, u: &mut Vertex) {
        // Checks that the inverse exists and gets a reference to it
        self.check_inverse();
        let inverse = self.inverse.as_ref().unwrap();

        // Initialises the output vertex
        let mut v = Vertex::zero();

        // Performs matrix multiplication
        for i in 0..4 {
            v[i as u8] = inverse[0][i] * u.x
                + inverse[1][i] * u.y
                + inverse[2][i] * u.z
                + inverse[3][i] * u.w;
        }

        // Updates original vertex with the transformed value
        *u = v;
    }

    /// Applies the transform to a vector in-place.
    pub fn apply_vector(&self, u: &mut Vector) {
        // Initialises output vector
        let mut v = Vector::zero();

        // Performs matrix multiplication
        for i in 0..3 {
            v[i as u8] =
                self.matrix[i][0] * u.x + self.matrix[i][1] * u.y + self.matrix[i][2] * u.z;
        }

        // Updates original vector with the transformed value
        *u = v;
    }

    /// Applies the transposed transform to a vector in-place.
    pub fn apply_transpose_vector(&self, u: &mut Vector) {
        // Initialises output vector
        let mut v = Vector::zero();

        // Performs matrix multiplication
        for i in 0..3 {
            v[i as u8] =
                self.matrix[0][i] * u.x + self.matrix[1][i] * u.y + self.matrix[2][i] * u.z;
        }

        // Updates original vector with the transformed value
        *u = v;
    }

    /// Applies the inverse transform to a vector in-place.
    pub fn apply_inverse_vector(&self, u: &mut Vector) {
        // Checks that the inverse exists and gets a reference to it
        self.check_inverse();
        let inverse = self.inverse.as_ref().unwrap();

        // Initialises output vector
        let mut v = Vector::zero();

        // Performs matrix multiplication
        for i in 0..3 {
            v[i as u8] = inverse[i][0] * u.x + inverse[i][1] * u.y + inverse[i][2] * u.z;
        }

        // Updates original vector with the transformed value
        *u = v;
    }

    /// Applies the transposed inverse transform to a vector in-place.
    pub fn apply_transpose_inverse_vector(&self, u: &mut Vector) {
        // Checks that the inverse exists and gets a reference to it
        self.check_inverse();
        let inverse = self.inverse.as_ref().unwrap();

        // Initialises output vector
        let mut v = Vector::zero();

        // Performs matrix multiplication
        for i in 0..3 {
            v[i as u8] = inverse[0][i] * u.x + inverse[1][i] * u.y + inverse[2][i] * u.z;
        }

        // Updates original vector with the transformed value
        *u = v;
    }

    /// Multiplies a vertex by the transform and returns a new vertex.
    pub fn mul_vertex(&self, mut u: Vertex) -> Vertex {
        self.apply_vertex(&mut u);
        u
    }

    /// Multiplies a vertex by the transposed transform and returns a new vertex.
    pub fn mul_transpose_vertex(&self, mut u: Vertex) -> Vertex {
        self.apply_transpose_vertex(&mut u);
        u
    }

    /// Multiplies a vector by the transform and returns a new vector.
    pub fn mul_vector(&self, mut u: Vector) -> Vector {
        self.apply_vector(&mut u);
        u
    }

    /// Multiplies a vector by the transposed transform and returns a new vector.
    pub fn mul_transpose_vector(&self, mut u: Vector) -> Vector {
        self.apply_transpose_vector(&mut u);
        u
    }

    /// Multiplies a vertex by the inverse transform and returns a new vertex.
    pub fn mul_inverse_vertex(&self, mut u: Vertex) -> Vertex {
        self.apply_inverse_vertex(&mut u);
        u
    }

    /// Multiplies a vertex by the transposed inverse transform and returns a new vertex.
    pub fn mul_transpose_inverse_vertex(&self, mut u: Vertex) -> Vertex {
        self.apply_transpose_inverse_vertex(&mut u);
        u
    }

    /// Multiplies a vector by the inverse transform and returns a new vector.
    pub fn mul_inverse_vector(&self, mut u: Vector) -> Vector {
        self.apply_inverse_vector(&mut u);
        u
    }

    /// Multiplies a vector by the transposed inverse transform and returns a new vector.
    pub fn mul_transpose_inverse_vector(&self, mut u: Vector) -> Vector {
        self.apply_transpose_inverse_vector(&mut u);
        u
    }

    /// Transposes the Transform matrix in-place.
    pub fn transpose(&mut self) {
        self.matrix = Self::transpose_matrix(self.matrix);
        if let Some(inverse) = self.inverse {
            self.inverse = Some(Self::transpose_matrix(inverse));
        }
    }

    /// Gets the transpose of the Transform as a new Transform.
    pub fn transposed(&self) -> Transform {
        let mut result = self.clone();
        result.transpose();
        result
    }

    /// Transposes a given matrix array.
    pub fn transpose_matrix(matrix: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
        let mut transposed = [[0.0; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                transposed[row][col] = matrix[col][row];
            }
        }
        transposed
    }

    /// Performs checks to ensure it is safe to unwrap and use `self.inverse`.
    fn check_inverse(&self) {
        // Checks that an attempt to calculate the inverse has been made
        if !self.inverse_calculated {
            panic!(
                "Attempted to get an inverse that had not yet been calculated. \n \
                Call `calculate_transform()` before getting the inverse, \
                or create the Transform with the `with_inverse` constructor."
            )
        }

        // Checks that the inverse exists (i.e. the matrix is non-singular)
        if self.inverse.is_none() {
            panic!("Attempted to get the inverse of a singular transform matrix.")
        }
    }

    /// Gets the inverse of the transform, returned as a new Transform object.
    pub fn get_inverse(&self) -> Transform {
        self.check_inverse();
        Transform {
            matrix: self.inverse.unwrap(),
            inverse: Some(self.matrix),
            inverse_calculated: true,
        }
    }

    /// Calculates the inverse of a given matrix, if one exists.
    pub fn calculate_inverse(&mut self) {
        self.inverse_calculated = true;
        let mut result = [[0.0; 4]; 4];

        // Calculates first column of adjugate matrix
        result[0][0] = self.matrix[1][1] * self.matrix[2][2] * self.matrix[3][3]
            - self.matrix[1][1] * self.matrix[2][3] * self.matrix[3][2]
            - self.matrix[2][1] * self.matrix[1][2] * self.matrix[3][3]
            + self.matrix[2][1] * self.matrix[1][3] * self.matrix[3][2]
            + self.matrix[3][1] * self.matrix[1][2] * self.matrix[2][3]
            - self.matrix[3][1] * self.matrix[1][3] * self.matrix[2][2];
        result[1][0] = -self.matrix[1][0] * self.matrix[2][2] * self.matrix[3][3]
            + self.matrix[1][0] * self.matrix[2][3] * self.matrix[3][2]
            + self.matrix[2][0] * self.matrix[1][2] * self.matrix[3][3]
            - self.matrix[2][0] * self.matrix[1][3] * self.matrix[3][2]
            - self.matrix[3][0] * self.matrix[1][2] * self.matrix[2][3]
            + self.matrix[3][0] * self.matrix[1][3] * self.matrix[2][2];
        result[2][0] = self.matrix[1][0] * self.matrix[2][1] * self.matrix[3][3]
            - self.matrix[1][0] * self.matrix[2][3] * self.matrix[3][1]
            - self.matrix[2][0] * self.matrix[1][1] * self.matrix[3][3]
            + self.matrix[2][0] * self.matrix[1][3] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[1][1] * self.matrix[2][3]
            - self.matrix[3][0] * self.matrix[1][3] * self.matrix[2][1];
        result[3][0] = -self.matrix[1][0] * self.matrix[2][1] * self.matrix[3][2]
            + self.matrix[1][0] * self.matrix[2][2] * self.matrix[3][1]
            + self.matrix[2][0] * self.matrix[1][1] * self.matrix[3][2]
            - self.matrix[2][0] * self.matrix[1][2] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[1][1] * self.matrix[2][2]
            + self.matrix[3][0] * self.matrix[1][2] * self.matrix[2][1];

        // Calculates determinant and early exits if no inverse exists
        let det = self.matrix[0][0] * result[0][0]
            + self.matrix[0][1] * result[1][0]
            + self.matrix[0][2] * result[2][0]
            + self.matrix[0][3] * result[3][0];

        if det == 0. {
            self.inverse = None;
            return;
        }

        // Calculates second column of adjugate matrix
        result[0][1] = -self.matrix[0][1] * self.matrix[2][2] * self.matrix[3][3]
            + self.matrix[0][1] * self.matrix[2][3] * self.matrix[3][2]
            + self.matrix[2][1] * self.matrix[0][2] * self.matrix[3][3]
            - self.matrix[2][1] * self.matrix[0][3] * self.matrix[3][2]
            - self.matrix[3][1] * self.matrix[0][2] * self.matrix[2][3]
            + self.matrix[3][1] * self.matrix[0][3] * self.matrix[2][2];
        result[1][1] = self.matrix[0][0] * self.matrix[2][2] * self.matrix[3][3]
            - self.matrix[0][0] * self.matrix[2][3] * self.matrix[3][2]
            - self.matrix[2][0] * self.matrix[0][2] * self.matrix[3][3]
            + self.matrix[2][0] * self.matrix[0][3] * self.matrix[3][2]
            + self.matrix[3][0] * self.matrix[0][2] * self.matrix[2][3]
            - self.matrix[3][0] * self.matrix[0][3] * self.matrix[2][2];
        result[2][1] = -self.matrix[0][0] * self.matrix[2][1] * self.matrix[3][3]
            + self.matrix[0][0] * self.matrix[2][3] * self.matrix[3][1]
            + self.matrix[2][0] * self.matrix[0][1] * self.matrix[3][3]
            - self.matrix[2][0] * self.matrix[0][3] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[0][1] * self.matrix[2][3]
            + self.matrix[3][0] * self.matrix[0][3] * self.matrix[2][1];
        result[3][1] = self.matrix[0][0] * self.matrix[2][1] * self.matrix[3][2]
            - self.matrix[0][0] * self.matrix[2][2] * self.matrix[3][1]
            - self.matrix[2][0] * self.matrix[0][1] * self.matrix[3][2]
            + self.matrix[2][0] * self.matrix[0][2] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[0][1] * self.matrix[2][2]
            - self.matrix[3][0] * self.matrix[0][2] * self.matrix[2][1];

        // Calculates third column of adjugate matrix
        result[0][2] = self.matrix[0][1] * self.matrix[1][2] * self.matrix[3][3]
            - self.matrix[0][1] * self.matrix[1][3] * self.matrix[3][2]
            - self.matrix[1][1] * self.matrix[0][2] * self.matrix[3][3]
            + self.matrix[1][1] * self.matrix[0][3] * self.matrix[3][2]
            + self.matrix[3][1] * self.matrix[0][2] * self.matrix[1][3]
            - self.matrix[3][1] * self.matrix[0][3] * self.matrix[1][2];
        result[1][2] = -self.matrix[0][0] * self.matrix[1][2] * self.matrix[3][3]
            + self.matrix[0][0] * self.matrix[1][3] * self.matrix[3][2]
            + self.matrix[1][0] * self.matrix[0][2] * self.matrix[3][3]
            - self.matrix[1][0] * self.matrix[0][3] * self.matrix[3][2]
            - self.matrix[3][0] * self.matrix[0][2] * self.matrix[1][3]
            + self.matrix[3][0] * self.matrix[0][3] * self.matrix[1][2];
        result[2][2] = self.matrix[0][0] * self.matrix[1][1] * self.matrix[3][3]
            - self.matrix[0][0] * self.matrix[1][3] * self.matrix[3][1]
            - self.matrix[1][0] * self.matrix[0][1] * self.matrix[3][3]
            + self.matrix[1][0] * self.matrix[0][3] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[0][1] * self.matrix[1][3]
            - self.matrix[3][0] * self.matrix[0][3] * self.matrix[1][1];
        result[3][2] = -self.matrix[0][0] * self.matrix[1][1] * self.matrix[3][2]
            + self.matrix[0][0] * self.matrix[1][2] * self.matrix[3][1]
            + self.matrix[1][0] * self.matrix[0][1] * self.matrix[3][2]
            - self.matrix[1][0] * self.matrix[0][2] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[0][1] * self.matrix[1][2]
            + self.matrix[3][0] * self.matrix[0][2] * self.matrix[1][1];

        // Calculates fourth column of adjugate matrix
        result[0][3] = -self.matrix[0][1] * self.matrix[1][2] * self.matrix[2][3]
            + self.matrix[0][1] * self.matrix[1][3] * self.matrix[2][2]
            + self.matrix[1][1] * self.matrix[0][2] * self.matrix[2][3]
            - self.matrix[1][1] * self.matrix[0][3] * self.matrix[2][2]
            - self.matrix[2][1] * self.matrix[0][2] * self.matrix[1][3]
            + self.matrix[2][1] * self.matrix[0][3] * self.matrix[1][2];
        result[1][3] = self.matrix[0][0] * self.matrix[1][2] * self.matrix[2][3]
            - self.matrix[0][0] * self.matrix[1][3] * self.matrix[2][2]
            - self.matrix[1][0] * self.matrix[0][2] * self.matrix[2][3]
            + self.matrix[1][0] * self.matrix[0][3] * self.matrix[2][2]
            + self.matrix[2][0] * self.matrix[0][2] * self.matrix[1][3]
            - self.matrix[2][0] * self.matrix[0][3] * self.matrix[1][2];
        result[2][3] = -self.matrix[0][0] * self.matrix[1][1] * self.matrix[2][3]
            + self.matrix[0][0] * self.matrix[1][3] * self.matrix[2][1]
            + self.matrix[1][0] * self.matrix[0][1] * self.matrix[2][3]
            - self.matrix[1][0] * self.matrix[0][3] * self.matrix[2][1]
            - self.matrix[2][0] * self.matrix[0][1] * self.matrix[1][3]
            + self.matrix[2][0] * self.matrix[0][3] * self.matrix[1][1];
        result[3][3] = self.matrix[0][0] * self.matrix[1][1] * self.matrix[2][2]
            - self.matrix[0][0] * self.matrix[1][2] * self.matrix[2][1]
            - self.matrix[1][0] * self.matrix[0][1] * self.matrix[2][2]
            + self.matrix[1][0] * self.matrix[0][2] * self.matrix[2][1]
            + self.matrix[2][0] * self.matrix[0][1] * self.matrix[1][2]
            - self.matrix[2][0] * self.matrix[0][2] * self.matrix[1][1];

        // Pre-computes determinant reciprocal for more efficient division
        let det_recip = 1.0 / det;

        // Divides every element in the adjugate matrix by the determinant
        for row in result.iter_mut() {
            for elem in row.iter_mut() {
                *elem *= det_recip;
            }
        }
        self.inverse = Some(result)
    }
}

/// Implements multiplication of one Transform by another.
impl Mul for Transform {
    type Output = Transform;

    fn mul(self, other: Transform) -> Transform {
        (&self).mul(&other)
    }
}

/// Implements multiplication of one Transform reference by another.
impl Mul for &Transform {
    type Output = Transform;

    fn mul(self, other: &Transform) -> Transform {
        let mut result = [[0.0; 4]; 4];
        for x in 0..4 {
            for y in 0..4 {
                result[x][y] = self.matrix[x][0] * other.matrix[0][y]
                    + self.matrix[x][1] * other.matrix[1][y]
                    + self.matrix[x][2] * other.matrix[2][y]
                    + self.matrix[x][3] * other.matrix[3][y];
            }
        }
        Transform::from_array(result)
    }
}

/// Implements multiplication of a vertex by a Transform.
impl Mul<Vertex> for Transform {
    type Output = Vertex;

    fn mul(self, other: Vertex) -> Vertex {
        self.mul_vertex(other)
    }
}

/// Implements multiplication of a vector by a Transform.
impl Mul<Vector> for Transform {
    type Output = Vector;

    fn mul(self, other: Vector) -> Vector {
        self.mul_vector(other)
    }
}

/// Implements indexing a transformation matrix to access its elements.
impl Index<usize> for Transform {
    type Output = [f64; 4];

    fn index(&self, i: usize) -> &[f64; 4] {
        &self.matrix[i]
    }
}

/// Implements loading transforms from a YAML file.
impl FromYaml for Transform {
    fn from_yaml(yaml: &Yaml) -> Result<Transform, YamlPropertyError> {
        // Checks that the transform is given as a hash map
        let hash = match yaml {
            Yaml::Hash(ref hash) => hash.clone(),
            _ => return Err(YamlPropertyError::invalid("type")),
        };

        // Checks that the transform only contains one key
        if hash.len() != 1 {
            return Err(YamlPropertyError::invalid("type"));
        };

        // Retrieves the key from the hash map
        let (key, _) = hash.iter().next().unwrap();

        if let Yaml::String(s) = key {
            match s.as_str() {
                // Allows reading translations
                "translate" => {
                    let t: Vector = parse_struct(yaml, "translate")?;
                    Ok(Transform::translate(t[0], t[1], t[2]))
                }

                // Allows reading rotations about the x-axis
                "rotate_x" => {
                    let s = parse_float(yaml, "rotate_x")?;
                    Ok(Transform::rotate_x(s))
                }

                // Allows reading rotations about the y-axis
                "rotate_y" => {
                    let s = parse_float(yaml, "rotate_y")?;
                    Ok(Transform::rotate_y(s))
                }

                // Allows reading rotations about the z-axis
                "rotate_z" => {
                    let s = parse_float(yaml, "rotate_z")?;
                    Ok(Transform::rotate_z(s))
                }

                // Allows reading scaling transforms
                "scale" => {
                    let s = parse_float(yaml, "scale")?;
                    Ok(Transform::scale(s))
                }

                // Allows reading stretches along the x-axis
                "stretch_x" => {
                    let s = parse_float(yaml, "stretch_x")?;
                    Ok(Transform::stretch_x(s))
                }

                // Allows reading stretches along the y-axis
                "stretch_y" => {
                    let s = parse_float(yaml, "stretch_y")?;
                    Ok(Transform::stretch_y(s))
                }

                // Allows reading stretches along the z-axis
                "stretch_z" => {
                    let s = parse_float(yaml, "stretch_z")?;
                    Ok(Transform::stretch_z(s))
                }

                // Allows reading an arbitrary transform matrix
                "matrix" => {
                    // Reads in the rows of the matrix
                    let rows = parse_vec(yaml, "matrix")?;

                    // Checks that the given matrix has 4 rows
                    if rows.len() != 4 {
                        return Err(YamlPropertyError::invalid("matrix"));
                    };

                    // Constructs transformation matrix
                    let mut matrix = [[0.0; 4]; 4];
                    for i in 0..4 {
                        // Attempts to parse the row as an array
                        let row = match &rows[i] {
                            Yaml::Array(v) => v.clone(),
                            _ => return Err(YamlPropertyError::invalid("matrix")),
                        };

                        // Converts the row array to floats
                        let row_values = parse_float_array(&row).extend_err("matrix")?;

                        // Copies row values into the matrix
                        matrix[i].copy_from_slice(&row_values);
                    }

                    // Constructs transform using the matrix array
                    Ok(Transform::from_array(matrix))
                }

                // Returns an error if an invalid transform identifier is given
                _ => Err(YamlPropertyError::invalid("type")),
            }
        } else {
            // Returns an error if the transform identifier is not a string
            Err(YamlPropertyError::invalid("type"))
        }
    }
}
