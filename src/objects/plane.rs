use crate::core::{Hit, Ray};
use crate::objects::ObjectTrait;
use crate::utils::yaml::{
    parse_float, parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, Vector, AABB};
use std::ops::Neg;
use yaml_rust::Yaml;

#[derive(Debug)]
/// An infinite surface that has volume on the negative side of the plane.
pub struct Plane {
    normal: Vector, // Normal of the plane
    d: f64,         // Plane constant

    transform: Transform, // Transform of the plane

    material: String, // Material of the plane
}

impl Plane {
    /// Creates a new Plane object, using construction based on the implicit plane representation:
    ///
    /// ax + by + cz + d = 0
    ///
    /// # Arguments
    ///
    /// * `a`: The x component of the plane's normal vector.
    /// * `b`: The y component of the plane's normal vector.
    /// * `c`: The z component of the plane's normal vector.
    /// * `d`: The plane constant.
    /// * `material`: Material of the plane.
    pub fn new(normal: Vector, d: f64, material: &str) -> Plane {
        Plane {
            d,
            normal: normal.unit(),
            material: String::from(material),
            transform: Transform::identity(),
        }
    }
}

impl ObjectTrait for Plane {
    fn intersection(&self, ray: &Ray) -> Vec<Hit<'_>> {
        // Transforms ray into the sphere's object space
        let ray = ray.to_object_space(&self.transform);

        // Calculates values used for intersection detection
        let u = Vector::dot(&self.normal, &ray.position.to_vector()) + self.d;
        let v = Vector::dot(&self.normal, &ray.direction);

        // Handles case where the ray is parallel to the plane
        if v.abs() == 0.0 {
            // Handles case where the ray is inside the plane
            if u < 0.0 {
                // Sets t values to +/- infinity
                let t0 = f64::MIN;
                let t1 = f64::MAX;

                // Calculates positions of the intersections
                let pos0 = ray.position + t0 * ray.direction;
                let pos1 = ray.position + t1 * ray.direction;

                // Calculates normal of the intersections
                let normal = if Vector::dot(&self.normal, &ray.direction) > 0. {
                    -self.normal
                } else {
                    self.normal
                };

                // Creates hit objects
                let mut hit0 = Hit::new(t0, true, pos0, normal, &self.material, None);
                let mut hit1 = Hit::new(t1, false, pos1, normal, &self.material, None);

                // Transforms the hits back into world space
                hit0.transform(&self.transform);
                hit1.transform(&self.transform);
                return vec![hit0, hit1];
            }
            return Vec::new();
        }

        // Calculates t value and point of intersection
        let t = u / -v;
        let pos = ray.position + t * ray.direction;

        // Calculates normal of the intersection
        let normal = if Vector::dot(&self.normal, &ray.direction) > 0. {
            self.normal.neg()
        } else {
            self.normal
        };

        // Creates hit objects
        let mut hit0;
        let mut hit1;
        if v > 0.0 {
            let inf_pos = ray.position + f64::MIN * ray.direction;
            hit0 = Hit::new(f64::MIN, true, inf_pos, normal, &self.material, None);
            hit1 = Hit::new(t, false, pos, normal, &self.material, None);
        } else {
            let inf_pos = ray.position + f64::MAX * ray.direction;
            hit0 = Hit::new(t, true, pos, normal, &self.material, None);
            hit1 = Hit::new(f64::MAX, false, inf_pos, normal, &self.material, None);
        }

        // Transforms the hits back into world space
        hit0.transform(&self.transform);
        hit1.transform(&self.transform);
        vec![hit0, hit1]
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.transform = transform * self.transform;
        self.transform.calculate_inverse();
    }

    fn get_bounding_box(&self) -> &AABB {
        unimplemented!("Planes are an infinite surface without a bounding box.")
    }
}

/// Implements loading a `Plane` from a YAML file.
impl FromYaml for Plane {
    fn from_yaml(yaml: &Yaml) -> Result<Plane, YamlPropertyError> {
        // Parses properties for the plane
        let normal = parse_struct(yaml, "normal")?;
        let d = parse_float(yaml, "d")?;
        let material = parse_string(yaml, "material")?;

        // Creates the plane instance
        let mut plane = Plane::new(normal, d, &material);

        // Applies any present transforms to the plane
        let transform = parse_transforms(yaml)?;
        plane.apply_transform(transform);

        Ok(plane)
    }
}
