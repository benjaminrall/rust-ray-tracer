use crate::core::{Hit, Ray};
use crate::objects::instances::Instances;
use crate::objects::{
    AABox, ConstantMedium, Group, Instance, KDTree, Plane, PolyMesh, Quad, Quadratic, Sphere,
    Triangle, CSG,
};
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use crate::utils::{Transform, AABB};
use enum_dispatch::enum_dispatch;
use yaml_rust::Yaml;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent all hittable objects.
pub enum Object {
    Sphere,
    Plane,
    PolyMesh,
    Triangle,
    Quad,
    AABox,
    KDTree,
    Quadratic,
    CSG,
    Group,
    Instance,
    Instances,
    ConstantMedium,
}

#[enum_dispatch(Object)]
/// Trait which must be implemented by all `Object` structs.
pub trait ObjectTrait {
    /// Returns all points of intersection, sorted by `t`, if the given ray intersects the object.
    fn intersection(&self, ray: &Ray) -> Vec<Hit>;

    /// Applies a transformation to the object.
    fn apply_transform(&mut self, transform: Transform);

    /// Gets the axis-aligned bounding box of the object.
    fn get_bounding_box(&self) -> &AABB;
}

/// Implements loading `Object` structs from a YAML file.
impl FromYaml for Object {
    fn from_yaml(yaml: &Yaml) -> Result<Object, YamlPropertyError> {
        // Parses object type as a String
        let object_type = parse_string(yaml, "type")?;

        // Matches the type to its respective object
        match object_type.as_str() {
            "Sphere" => Ok(Sphere::from_yaml(yaml)?.into()),
            "Plane" => Ok(Plane::from_yaml(yaml)?.into()),
            "Quad" => Ok(Quad::from_yaml(yaml)?.into()),
            "AABox" => Ok(AABox::from_yaml(yaml)?.into()),
            "Quadratic" => Ok(Quadratic::from_yaml(yaml)?.into()),
            "CSG" => Ok(CSG::from_yaml(yaml)?.into()),
            "Group" => Ok(Group::from_yaml(yaml)?.into()),
            "PolyMesh" => Ok(PolyMesh::from_yaml(yaml)?.into()),
            "Instances" => Ok(Instances::from_yaml(yaml)?.into()),
            "KDTree" => Ok(KDTree::from_yaml(yaml)?.into()),
            "ConstantMedium" => Ok(ConstantMedium::from_yaml(yaml)?.into()),
            _ => Err(YamlPropertyError::invalid("type")),
        }
    }
}
