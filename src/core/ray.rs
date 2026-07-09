use crate::utils::{Transform, Vector, Vertex};
use crate::EPSILON;

#[derive(Debug)]
/// Struct to store and manipulate 3D rays.
pub struct Ray {
    pub position: Vertex,  // Origin position of the ray
    pub direction: Vector, // Direction of the ray

    pub inside: bool, // Flag for whether the ray is inside an object, used for constant mediums
}

impl Ray {
    /// Creates a new `Ray` instance with a given origin position and direction.
    pub fn new(position: Vertex, direction: Vector) -> Ray {
        Ray {
            position,
            direction,
            inside: false,
        }
    }

    /// Creates a new `Ray` instance, offset slightly by its direction for numerical stability.
    pub fn offset(position: Vertex, direction: Vector) -> Ray {
        Ray {
            position: position + EPSILON * direction,
            direction,
            inside: false,
        }
    }

    /// Creates a new `Ray` instance, offset slightly by its direction,
    /// and including a tag for whether it's inside the object.
    pub fn offset_inside(position: Vertex, direction: Vector, inside: bool) -> Ray {
        Ray {
            position: position + EPSILON * direction,
            direction,
            inside,
        }
    }

    /// Transforms a ray into the object space of a given transformation matrix.
    pub fn to_object_space(&self, transform: &Transform) -> Ray {
        // Calculates the new ray's parameters
        let new_pos = transform.mul_inverse_vertex(self.position);
        let new_dir = transform.mul_inverse_vector(self.direction);
        Ray::new(new_pos, new_dir)
    }
}
