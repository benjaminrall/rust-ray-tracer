use crate::drawing::TexCoords;
use crate::utils::{Transform, Vector, Vertex};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
/// Struct to store information about an intersection between a ray and an object.
pub struct Hit<'a> {
    pub t: f64,         // The `t` value at which the ray hit the object
    pub entering: bool, // Whether the ray was entering the object

    pub position: Vertex, // The position of the hit on the surface of the object
    pub normal: Vector,   // The normal to the object's surface at the position of the hit

    pub material: &'a str, // The material of the hit object, as a string reference to the scene's material map

    pub tex_coords: Option<TexCoords>, // Optional texture coordinates of the hit
}

impl<'a> Hit<'_> {
    /// Creates a new `Hit` instance.
    ///
    /// # Arguments
    ///
    /// * `t`: The t value at which the ray hit the object.
    /// * `entering`: Whether the ray was entering the object.
    /// * `position`: The position of the hit on the surface of the object.
    /// * `normal`: The normal to the object's surface at the position of the hit.
    /// * `material`: String reference to the material of the hit object.
    /// * `tex_coords`: Optional texture coordinates of the hit.
    pub fn new(
        t: f64,
        entering: bool,
        position: Vertex,
        normal: Vector,
        material: &'a str,
        tex_coords: Option<TexCoords>,
    ) -> Hit<'a> {
        Hit {
            t,
            entering,
            material,
            position,
            normal,
            tex_coords,
        }
    }

    /// Transforms the hit using the given transformation.
    pub fn transform(&mut self, transform: &Transform) {
        // Calculates the new hit point and normal
        transform.apply_vertex(&mut self.position);
        transform.apply_transpose_inverse_vector(&mut self.normal);

        // Ensures the normal remains normalised
        self.normal.normalise()
    }
}

/// Implements equating hits based on their `t` value.
impl PartialEq for Hit<'_> {
    fn eq(&self, other: &Hit) -> bool {
        self.t == other.t
    }
}

/// Implements comparing hits based on their `t` value.
impl PartialOrd for Hit<'_> {
    fn partial_cmp(&self, other: &Hit) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}
