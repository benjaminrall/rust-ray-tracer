use crate::core::{Hit, Ray};
use crate::materials::{LambertianMaterial, VolumeMaterial};
use crate::objects::{Object, ObjectTrait};
use crate::utils::yaml::{
    parse_float, parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{random_float, Transform, Vector, AABB};
use crate::EPSILON;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Struct representing a constant scattering medium.
pub struct ConstantMedium {
    volume: Box<Object>, // Hittable object representing the volume of the constant medium
    material: String,    // Material of the medium - should be a volume based material.

    neg_density_recip: f64, // Store of the negative reciprocal of density for volume calculations
}

impl ConstantMedium {
    /// Creates a new constant medium with the given volume, density, and material.
    pub fn new(volume: Object, density: f64, material: &str) -> ConstantMedium {
        ConstantMedium {
            volume: Box::new(volume),
            material: String::from(material),
            neg_density_recip: -1.0 / density,
        }
    }
}

impl ObjectTrait for ConstantMedium {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Checks that the ray is not inside an object
        if ray.inside {
            return Vec::new();
        }

        // Get the hit points with the surrounding volume
        let mut hits: Vec<Hit> = self.volume.intersection(ray);

        // Catches case where there are not two hits for the volume
        if hits.len() < 2 {
            return Vec::new();
        }

        // If the ray is inside the volume, shifts the first hit position to the ray's origin
        if hits[0].t < EPSILON {
            hits[0].t = EPSILON;
        }

        // Assigns the second hit as the last hit returned
        let hit2 = &hits[hits.len() - 1];

        // Catches the case where both hits lie before the volume
        if hit2.t < 0.0 {
            return Vec::new();
        }

        // Calculates the distance to a hit based on the medium's density
        let ray_length = ray.direction.length();
        let distance_inside_volume = (hit2.t - hits[0].t) * ray_length;
        let hit_distance = self.neg_density_recip * random_float().ln();

        // If the hit lies outside the volume, no hits are returned
        if hit_distance > distance_inside_volume {
            return Vec::new();
        }

        // Otherwise, the `t` value and position of the hit are calculated and used to construct a hit record
        let t = hits[0].t + hit_distance / ray_length;
        let pos = ray.position + t * ray.direction;
        vec![Hit::new(t, true, pos, Vector::zero(), &self.material, None)]
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.volume.apply_transform(transform)
    }

    fn get_bounding_box(&self) -> &AABB {
        self.volume.get_bounding_box()
    }
}

/// Implements loading a `ConstantMedium` from a YAML file.
impl FromYaml for ConstantMedium {
    fn from_yaml(yaml: &Yaml) -> Result<ConstantMedium, YamlPropertyError> {
        // Parses properties for the material
        let volume = parse_struct(yaml, "volume")?;
        let density = parse_float(yaml, "density")?;
        let material = parse_string(yaml, "material")?;

        // Creates the medium instance
        let mut medium = ConstantMedium::new(volume, density, &material);

        // Applies any present transforms to the medium
        let transform = parse_transforms(yaml)?;
        medium.apply_transform(transform);

        Ok(medium)
    }
}
