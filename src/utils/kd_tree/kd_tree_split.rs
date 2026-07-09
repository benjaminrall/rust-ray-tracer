use crate::core::{Hit, Ray};
use crate::objects::Object;
use crate::utils::kd_tree::{EventPlane, KDTreeNode, KDTreeNodeTrait};
use crate::utils::{merge, Vector, AABB};

#[derive(Debug)]
/// Struct to represent a split node in a KDTree.
pub struct KDTreeSplit {
    left: Box<KDTreeNode>,  // Node left of the split plane
    right: Box<KDTreeNode>, // Node right of the split plane

    bounding_box: AABB, // Bounding box of the area covered by the node
    split: EventPlane,  // Plane at which the bounding box was split
}

impl KDTreeSplit {
    /// Creates a new split node for a KDTree.
    ///
    /// # Arguments
    ///
    /// * `left`: Node to the left of the split plane.
    /// * `right`: Node to the right of the split plane.
    /// * `bounding_box`: Bounding box of the area covered by the node.
    /// * `split`: Plane at which the bounding box was split.
    pub fn new(
        left: KDTreeNode,
        right: KDTreeNode,
        bounding_box: AABB,
        split: EventPlane,
    ) -> KDTreeSplit {
        KDTreeSplit {
            left: Box::new(left),
            right: Box::new(right),
            bounding_box,
            split,
        }
    }
}

impl KDTreeNodeTrait for KDTreeSplit {
    fn intersection<'a>(&self, ray: &Ray, objects: &'a Vec<Object>) -> Vec<Hit<'a>> {
        // Sets initial start and end values for the interval
        let mut start = f64::MIN;
        let mut end = f64::MAX;

        // Calculates the reciprocal of the ray's direction in each dimension
        let direction_recip = Vector::new(
            1. / ray.direction.x,
            1. / ray.direction.y,
            1. / ray.direction.z,
        );

        // Calculate intersection distances for each dimension
        for axis in 0..3 {
            if ray.direction[axis] == 0. {
                // If ray is parallel and outside the bounding box, there's no intersection
                if ray.position[axis] < self.bounding_box.get_min(axis)
                    || ray.position[axis] > self.bounding_box.get_max(axis)
                {
                    return Vec::new();
                }
                continue;
            }

            // Calculates intersection of the ray with the bounding box in the current axis
            let t0 = (self.bounding_box.get_min(axis) - ray.position[axis]) * direction_recip[axis];
            let t1 = (self.bounding_box.get_max(axis) - ray.position[axis]) * direction_recip[axis];

            // Calculates the start and end t values at which the ray enters the node's bounding box.
            start = start.max(t0.min(t1));
            end = end.min(t0.max(t1));
        }

        // No intersection when the intervals don't overlap
        if start > end || end < 0. {
            return Vec::new();
        }

        // Calculate intersection with split plane
        let t = if ray.direction[self.split.axis] == 0. {
            // Handle case where ray is parallel to the split plane
            if ray.position[self.split.axis] <= self.split.position {
                f64::MAX // Left of the split plane
            } else {
                f64::MIN // Right of the split plane
            }
        } else {
            (self.split.position - ray.position[self.split.axis]) * direction_recip[self.split.axis]
        };

        // Orient sides based on the ray's direction
        let sides = if ray.direction[self.split.axis] >= 0. {
            [&self.left, &self.right]
        } else {
            [&self.right, &self.left]
        };

        // Intersect with relevant sides and return hits
        if t <= start {
            sides[1].intersection(ray, objects)
        } else if t >= end {
            sides[0].intersection(ray, objects)
        } else {
            let left_hits = sides[0].intersection(ray, objects);
            let right_hits = sides[1].intersection(ray, objects);
            merge(left_hits, right_hits)
        }
    }

    fn get_bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
