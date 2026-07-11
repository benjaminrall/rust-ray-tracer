use crate::core::{Hit, Ray};
use crate::objects::{Object, ObjectTrait};
use crate::utils::{Transform, AABB};
use std::sync::Arc;

#[derive(Debug)]
/// Object that represents a transformed instance of another object.
pub struct Instance {
    object: Arc<Object>, // Source object of the instance

    transform: Transform, // Transform of the instance
    bounding_box: AABB,   // Bounding box of the instance
}

impl Instance {
    /// Creates a new instance of an object with a given transform.
    pub fn new(object: Arc<Object>, mut transform: Transform) -> Instance {
        transform.calculate_inverse();
        let bounding_box = object.get_bounding_box().transform(&transform);
        Instance {
            object,
            transform,
            bounding_box,
        }
    }
}

impl ObjectTrait for Instance {
    fn intersection(&self, ray: &Ray) -> Vec<Hit<'_>> {
        // Transforms ray into the tree's object space
        let ray = ray.to_object_space(&self.transform);

        // Intersects the transformed ray with the tree
        let mut hits = self.object.intersection(&ray);

        // Transforms the hits back into world space
        hits.iter_mut().for_each(|h| h.transform(&self.transform));

        hits
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.bounding_box = self.bounding_box.transform(&transform);
        self.transform = transform * self.transform;
        self.transform.calculate_inverse();
    }

    fn get_bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
