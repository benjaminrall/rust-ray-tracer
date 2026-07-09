use crate::utils::{Transform, Vector, Vertex};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Debug, Copy, Clone)]
/// Struct to store and manipulate axis-aligned bounding boxes.
pub struct AABB {
    // Minimum values
    min_x: f64,
    min_y: f64,
    min_z: f64,

    // Maximum values
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

impl AABB {
    /// Creates a new bounding box from given minimum and maximum values.
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> AABB {
        AABB {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    /// Creates a new bounding box from a minimum and maximum vector.
    pub fn from_vectors(minimum: Vector, maximum: Vector) -> AABB {
        AABB::new(
            minimum.x, minimum.y, minimum.z, maximum.x, maximum.y, maximum.z,
        )
    }

    /// Creates a new bounding box for a list of points.
    pub fn from_points(points: Vec<Vertex>) -> AABB {
        // Creates points iterator
        let mut iter = points.iter();

        // Assigns initial min/max values to the first point
        let first_point = match iter.next() {
            None => return AABB::empty(),
            Some(point) => point,
        };
        let (mut min_x, mut min_y, mut min_z) = (first_point.x, first_point.y, first_point.z);
        let (mut max_x, mut max_y, mut max_z) = (first_point.x, first_point.y, first_point.z);

        // Updates min and maxes for each point
        for point in iter {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            min_z = min_z.min(point.z);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
            max_z = max_z.max(point.z);
        }

        // Constructs AABB
        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Creates an empty bounding box, for combining many bounding boxes together.
    pub fn empty() -> AABB {
        AABB::new(f64::MAX, f64::MAX, f64::MAX, f64::MIN, f64::MIN, f64::MIN)
    }

    /// Computes the union of two bounding boxes and returns as a new bounding box.
    pub fn union(&self, other: &AABB) -> AABB {
        AABB::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.min_z.min(other.min_z),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
            self.max_z.max(other.max_z),
        )
    }

    /// Computes the union of two bounding boxes and assigns it to self.
    pub fn union_assign(&mut self, other: &AABB) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.min_z = self.min_z.min(other.min_z);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
        self.max_z = self.max_z.max(other.max_z);
    }

    /// Adds a point to the bounding box.
    pub fn add_point(&mut self, other: &Vertex) {
        self.min_x = self.min_x.min(other.x);
        self.min_y = self.min_y.min(other.y);
        self.min_z = self.min_z.min(other.z);
        self.max_x = self.max_x.max(other.x);
        self.max_y = self.max_y.max(other.y);
        self.max_z = self.max_z.max(other.z);
    }

    /// Calculates the surface area of a bounding box.
    pub fn surface_area(&self) -> f64 {
        let x_length = self.max_x - self.min_x;
        let y_length = self.max_y - self.min_y;
        let z_length = self.max_z - self.min_z;
        2.0 * (x_length * (y_length + z_length) + y_length * z_length)
    }

    /// Computes the intersection of two bounding boxes and assigns it to self.
    pub fn intersect_assign(&mut self, other: &AABB) {
        self.min_x = self.min_x.max(other.min_x);
        self.min_y = self.min_y.max(other.min_y);
        self.min_z = self.min_z.max(other.min_z);
        self.max_x = self.max_x.min(other.max_x);
        self.max_y = self.max_y.min(other.max_y);
        self.max_z = self.max_z.min(other.max_z);
    }

    /// Splits a bounding box along a given axis-aligned plane.
    ///
    /// # Arguments
    ///
    /// * `plane`: A tuple representing the plane, containing the axis to split along
    ///            and the position at which to split.
    ///
    /// returns: (AABB, AABB), the bounding boxes left and right of the split point respectively.
    pub fn split(&self, plane: (u8, f64)) -> (AABB, AABB) {
        let (axis, position) = plane;

        match axis {
            // Handles splits in the x dimension
            0 => (
                AABB::new(
                    self.min_x, self.min_y, self.min_z, position, self.max_y, self.max_z,
                ),
                AABB::new(
                    position, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
                ),
            ),

            // Handles splits in the y dimension
            1 => (
                AABB::new(
                    self.min_x, self.min_y, self.min_z, self.max_x, position, self.max_z,
                ),
                AABB::new(
                    self.min_x, position, self.min_z, self.max_x, self.max_y, self.max_z,
                ),
            ),

            // Handles splits in the z dimension
            2 => (
                AABB::new(
                    self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, position,
                ),
                AABB::new(
                    self.min_x, self.min_y, position, self.max_x, self.max_y, self.max_z,
                ),
            ),

            _ => panic!("Unsupported bounding box dimension: {}", axis),
        }
    }

    /// Computes the intersection of two bounding boxes and returns as a new bounding box.
    pub fn intersect(&self, other: &AABB) -> AABB {
        AABB::new(
            self.min_x.max(other.min_x),
            self.min_y.max(other.min_y),
            self.min_z.max(other.min_z),
            self.max_x.min(other.max_x),
            self.max_y.min(other.max_y),
            self.max_z.min(other.max_z),
        )
    }

    /// Gets the minimum value of the given dimension.
    pub fn get_min(&self, dimension: u8) -> f64 {
        match dimension {
            0 => self.min_x,
            1 => self.min_y,
            2 => self.min_z,
            _ => panic!("Unsupported bounding box dimension: {}", dimension),
        }
    }

    /// Gets the maximum value of the given dimension.
    pub fn get_max(&self, dimension: u8) -> f64 {
        match dimension {
            0 => self.max_x,
            1 => self.max_y,
            2 => self.max_z,
            _ => panic!("Unsupported bounding box dimension: {}", dimension),
        }
    }

    /// Returns a list of the bounding box's corner points.
    pub fn get_points(&self) -> [Vertex; 8] {
        [
            Vertex::new(self.min_x, self.min_y, self.min_z),
            Vertex::new(self.min_x, self.min_y, self.max_z),
            Vertex::new(self.min_x, self.max_y, self.min_z),
            Vertex::new(self.min_x, self.max_y, self.max_z),
            Vertex::new(self.max_x, self.min_y, self.min_z),
            Vertex::new(self.max_x, self.min_y, self.max_z),
            Vertex::new(self.max_x, self.max_y, self.min_z),
            Vertex::new(self.max_x, self.max_y, self.max_z),
        ]
    }

    /// Returns the bounding box transformed by the given Transform.
    pub fn transform(&self, transform: &Transform) -> AABB {
        let mut points = self.get_points();
        for point in points.iter_mut() {
            transform.apply_vertex(point);
        }
        AABB::from_points(points.to_vec())
    }
}

/// Implements calculating the union of bounding boxes using the '|' operator.
impl BitOr for AABB {
    type Output = AABB;

    fn bitor(self, other: AABB) -> AABB {
        self.union(&other)
    }
}

/// Implements calculating and assigning the union of bounding boxes using the '|=' operator.
impl BitOrAssign for AABB {
    fn bitor_assign(&mut self, other: AABB) {
        self.union_assign(&other)
    }
}

/// Implements calculating the intersection of bounding boxes using the '&' operator.
impl BitAnd for AABB {
    type Output = AABB;

    fn bitand(self, other: AABB) -> AABB {
        self.intersect(&other)
    }
}

/// Implements calculating and assigning the intersection of bounding boxes using the '&=' operator.
impl BitAndAssign for AABB {
    fn bitand_assign(&mut self, other: AABB) {
        self.intersect_assign(&other)
    }
}
