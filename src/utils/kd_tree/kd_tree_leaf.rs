use crate::core::{Hit, Ray};
use crate::objects::{Object, ObjectTrait};
use crate::utils::kd_tree::KDTreeNodeTrait;
use crate::utils::AABB;

#[derive(Debug)]
/// Struct to represent a leaf node in a KDTree.
pub struct KDTreeLeaf {
    object_indices: Vec<usize>, // Indices of the objects held by the leaf node
    bounding_box: AABB,         // Bounding box of the area covered by the node
}

impl KDTreeLeaf {
    /// Creates a new leaf node for a KDTree.
    ///
    /// # Arguments
    ///
    /// * `object_indices`: Indices of the objects held by the leaf node.
    /// * `bounding_box`: Bounding box of the area covered by the node
    pub fn new(object_indices: Vec<usize>, bounding_box: AABB) -> KDTreeLeaf {
        KDTreeLeaf {
            object_indices,
            bounding_box,
        }
    }
}

impl KDTreeNodeTrait for KDTreeLeaf {
    fn intersection<'a>(&self, ray: &Ray, objects: &'a Vec<Object>) -> Vec<Hit<'a>> {
        let mut hits = Vec::new();

        // Iterates over objects held by the node and stores any positive hits
        for &i in self.object_indices.iter() {
            hits.extend(
                objects[i]
                    .intersection(ray)
                    .into_iter()
                    .filter(|h| h.t > 0.),
            )
        }

        // Sorts and returns any hits
        hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        hits
    }

    fn get_bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
