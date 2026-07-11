use crate::core::{Hit, Ray};
use crate::objects::{Object, ObjectTrait};
use crate::utils::yaml::{parse_struct_array, parse_transforms, FromYaml, YamlPropertyError};
use crate::utils::{Transform, Vector, AABB};
use yaml_rust::Yaml;

/// Struct to group objects together with a single bounding box and unified transforms.
#[derive(Debug)]
pub struct Group {
    objects: Vec<Object>, // Objects stored within the group

    transform: Transform, // Transform of the group
    bounding_box: AABB,   // Bounding box of the group
}

impl Group {
    /// Creates a new group from a given list of objects.
    pub fn new(objects: Vec<Object>) -> Group {
        let bounding_box = objects
            .iter()
            .fold(AABB::empty(), |b, i| b.union(&i.get_bounding_box()));

        Group {
            objects,
            transform: Transform::identity(),
            bounding_box,
        }
    }
}

impl ObjectTrait for Group {
    fn intersection(&self, ray: &Ray) -> Vec<Hit<'_>> {
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

        // No intersection with the bounding box when the intervals don't overlap, so can early exit
        if start > end || end < 0. {
            return Vec::new();
        }

        // Transforms ray into the object space of the collection
        let ray = ray.to_object_space(&self.transform);

        // Iterates over the objects in the collection and stores all hits
        let mut hits = Vec::new();
        for instance in self.objects.iter() {
            hits.extend(instance.intersection(&ray))
        }

        // Transforms the hits back into world space
        hits.iter_mut().for_each(|h| h.transform(&self.transform));

        // Sorts the hits
        hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
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

/// Implements loading a `Group` from a YAML file.
impl FromYaml for Group {
    fn from_yaml(yaml: &Yaml) -> Result<Group, YamlPropertyError> {
        // Parses the group's objects and creates the group instance
        let objects = parse_struct_array(yaml, "objects")?;
        let mut group = Group::new(objects);

        // Applies any present transforms to the group
        let transform = parse_transforms(yaml)?;
        group.apply_transform(transform);

        Ok(group)
    }
}
