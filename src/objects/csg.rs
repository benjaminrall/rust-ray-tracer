use crate::core::{Hit, Ray};
use crate::objects::csg::CSGAction::*;
use crate::objects::{Object, ObjectTrait};
use crate::utils::yaml::{
    parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, AABB};
use yaml_rust::Yaml;

#[derive(Copy, Clone, Debug)]
/// Set operations supported by the CSG.
pub enum CSGOp {
    Union = 0,
    Intersect = 1,
    Difference = 2,
}

/// Actions that the CSG can take at each step in its intersection calculation.
pub enum CSGAction {
    AEnter,
    AExit,
    ADrop,
    BEnter,
    BExit,
    BDrop,
}

#[derive(Debug)]
/// Struct for Constructive Solid Geometry (CSG), that allows objects to be combined in a tree using
/// set operations on their volumes.
pub struct CSG {
    operation: CSGOp, // Operation for the CSG to use on the objects

    a: Box<Object>, // Left object in the tree
    b: Box<Object>, // Right object in the tree

    transform: Transform, // Transform of the CSG
}

impl CSG {
    /// Actions defined for each CSG operation.
    const ACTIONS: [[CSGAction; 8]; 3] = [
        [ADrop, BDrop, AExit, BDrop, ADrop, BExit, AEnter, BEnter],
        [AExit, BExit, ADrop, BEnter, AEnter, BDrop, ADrop, BDrop],
        [ADrop, BEnter, AExit, BExit, ADrop, BDrop, AEnter, BDrop],
    ];

    /// Applies a set operation to two given objects `a` and `b`.
    /// This gives the following possible combinations:
    /// - a ∪ b
    /// - a ∩ b
    /// - a \ b
    ///
    /// # Arguments
    ///
    /// * `a`: Left side object for the operation.
    /// * `operation`: Operation to be carried out.
    /// * `b`: Right side object for the operation.
    pub fn new(a: Object, operation: CSGOp, b: Object) -> CSG {
        CSG {
            operation,
            a: Box::new(a),
            b: Box::new(b),
            transform: Transform::identity(),
        }
    }
}

impl ObjectTrait for CSG {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Transforms ray into the sphere's object space
        let ray = ray.to_object_space(&self.transform);

        // Gets the hits by intersecting the child with the child objects
        let mut a_hits = self.a.intersection(&ray);
        let mut b_hits = self.b.intersection(&ray);

        // Reverses the returned hits, so their closest hit is at the end of the list
        a_hits.reverse();
        b_hits.reverse();

        // Sets initial index for both lists
        let mut ai = a_hits.len() as i32 - 1;
        let mut bi = b_hits.len() as i32 - 1;

        // Gets the operation being performed by the CSG, for indexing the ACTIONS list
        let op = self.operation as usize;

        // Builds the result hit list
        let mut result = vec![];
        while ai >= 0 && bi >= 0 {
            let j = ai as usize;
            let k = bi as usize;

            // Calculates the current state to determine the next action
            let mut state = 0;
            if a_hits[j].entering {
                state += 4
            }
            if b_hits[k].entering {
                state += 2
            }
            if a_hits[j].t > b_hits[k].t {
                state += 1
            }

            // Performs the next action on the hit lists
            match Self::ACTIONS[op][state] {
                // Adds an entering hit for an intersection with object A
                AEnter => {
                    let mut a_hit = a_hits.pop().unwrap();
                    a_hit.entering = true;
                    result.push(a_hit);
                    ai -= 1;
                }

                // Adds an exiting hit for an intersection with object A
                AExit => {
                    let mut a_hit = a_hits.pop().unwrap();
                    a_hit.entering = false;
                    result.push(a_hit);
                    ai -= 1;
                }

                // Drops an intersection with object A that doesn't contribute to the CSG volume
                ADrop => {
                    a_hits.pop().unwrap();
                    ai -= 1;
                }

                // Adds an entering hit for an intersection with object B
                BEnter => {
                    let mut b_hit = b_hits.pop().unwrap();
                    b_hit.entering = true;
                    result.push(b_hit);
                    bi -= 1;
                }

                // Adds an exiting hit for an intersection with object B
                BExit => {
                    let mut b_hit = b_hits.pop().unwrap();
                    b_hit.entering = false;
                    result.push(b_hit);
                    bi -= 1;
                }

                // Drops an intersection with object B that doesn't contribute to the CSG volume
                BDrop => {
                    b_hits.pop().unwrap();
                    bi -= 1;
                }
            }
        }

        // Handles any remaining hits after merging
        match self.operation {
            // For the union operation, add all remaining hits
            CSGOp::Union => {
                result.extend(a_hits.into_iter().rev());
                result.extend(b_hits.into_iter().rev());
            }

            // For the intersect operation, ignore remaining hits
            CSGOp::Intersect => {}

            // For the difference operation, add only remaining hits from object A
            CSGOp::Difference => {
                result.extend(a_hits.into_iter().rev());
            }
        }

        // Transforms the hits back into world space
        result.iter_mut().for_each(|h| h.transform(&self.transform));

        result
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.transform = transform * self.transform;
        self.transform.calculate_inverse();
    }

    fn get_bounding_box(&self) -> &AABB {
        todo!("CSGs can contain infinite surfaces, and therefore cannot yet have a bounding box.")
    }
}

/// Implements loading a `CSG` from a YAML file.
impl FromYaml for CSG {
    fn from_yaml(yaml: &Yaml) -> Result<CSG, YamlPropertyError> {
        // Parses properties for the CSG
        let left = parse_struct(yaml, "left")?;
        let right = parse_struct(yaml, "right")?;
        let op_string = parse_string(yaml, "op")?;

        // Converts operation string into the corresponding CSGOp type
        let op = match op_string.as_str() {
            "union" => CSGOp::Union,
            "intersect" => CSGOp::Intersect,
            "difference" => CSGOp::Difference,
            _ => return Err(YamlPropertyError::invalid("op")),
        };

        // Creates the CSG instance
        let mut csg = CSG::new(left, op, right);

        // Applies any present transforms to the CSG
        let transform = parse_transforms(yaml)?;
        csg.apply_transform(transform);

        Ok(csg)
    }
}
