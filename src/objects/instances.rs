use crate::core::{Hit, Ray};
use crate::objects::{Instance, Object, ObjectTrait};
use crate::utils::yaml::{
    parse_struct, parse_transforms, parse_vec, ExtendYamlResult, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, AABB};
use std::sync::Arc;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Object to represent a collection of instances of a base object.
pub struct Instances {
    instances: Vec<Instance>,
    transform: Transform,
    bounding_box: AABB,
}

impl Instances {
    /// Creates a new instances collection from a given list of instance objects.
    pub fn new(instances: Vec<Instance>) -> Instances {
        let bounding_box = instances
            .iter()
            .fold(AABB::empty(), |b, i| b.union(&i.get_bounding_box()));

        Instances {
            instances,
            transform: Transform::identity(),
            bounding_box,
        }
    }
}

impl ObjectTrait for Instances {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Transforms ray into the instances' object space
        let ray = ray.to_object_space(&self.transform);

        // Iterates over the instances in the collection and stores all hits
        let mut hits = Vec::new();
        for instance in self.instances.iter() {
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

/// Implements loading an `Instances` collection from a YAML file.
impl FromYaml for Instances {
    fn from_yaml(yaml: &Yaml) -> Result<Instances, YamlPropertyError> {
        // Parses properties for the instances collection
        let base: Arc<Object> = Arc::new(parse_struct(yaml, "base")?);
        let instances_yaml = parse_vec(yaml, "instances")?;

        // Creates all instance objects from the given instance transforms
        let mut instances = Vec::with_capacity(instances_yaml.len());
        for (i, instance_yaml) in instances_yaml.iter().enumerate() {
            let transform =
                parse_transforms(instance_yaml).extend_err(format!("instances.{}", i).as_str())?;
            instances.push(Instance::new(base.clone(), transform).into());
        }

        let mut instances = Instances::new(instances);

        // Applies any present transforms to the instances collection
        let transform = parse_transforms(yaml)?;
        instances.apply_transform(transform);

        Ok(instances)
    }
}
