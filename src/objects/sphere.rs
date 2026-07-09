use crate::core::{Hit, Ray};
use crate::drawing::TexCoords;
use crate::objects::object::ObjectTrait;
use crate::utils::yaml::{
    parse_float, parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, Vector, Vertex, AABB};
use crate::EPSILON;
use std::f64::consts::{FRAC_1_PI, PI};
use std::ops::Neg;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Simple Sphere primitive.
pub struct Sphere {
    centre: Vertex, // Centre of the sphere
    radius: f64,    // Radius of the sphere

    bounding_box: AABB,   // Bounding box of the sphere
    transform: Transform, // Transform of the sphere

    material: String, // Material of the sphere
}

impl Sphere {
    /// Creates a new Sphere object with a given centre point, radius, and material.
    pub fn new(centre: Vertex, radius: f64, material: &str) -> Sphere {
        let minimum = Vector::new(centre.x - radius, centre.y - radius, centre.z - radius);
        let maximum = Vector::new(centre.x + radius, centre.y + radius, centre.z + radius);
        let bounding_box = AABB::from_vectors(minimum, maximum);
        Sphere {
            centre,
            radius,
            material: String::from(material),
            bounding_box,
            transform: Transform::identity(),
        }
    }

    /// Gets the texture coordinates of a given position on the surface of the sphere.
    fn get_tex_coords(surface_pos: Vector) -> TexCoords {
        let u = (f64::atan2(-surface_pos.z, surface_pos.x) + PI) * 0.5 * FRAC_1_PI;
        let v = f64::acos(-surface_pos.y) * FRAC_1_PI;
        TexCoords::new(u, v, 0.0)
    }
}

impl ObjectTrait for Sphere {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Transforms ray into the sphere's object space
        let ray = ray.to_object_space(&self.transform);

        // Calculates vector from the centre of the sphere to the ray's position
        let ro = ray.position - self.centre;

        // Calculates quadratic equation coefficients
        let a = ray.direction.len_sqr();
        let half_b = Vector::dot(&ray.direction, &ro);
        let c = ro.len_sqr() - self.radius * self.radius;

        // Calculates the discriminant of the quadratic equation
        let discriminant = half_b * half_b - a * c;

        // Returns if there are no solutions
        if discriminant <= EPSILON {
            return Vec::new();
        }

        // Calculates t values of the intersection points
        let ds = f64::sqrt(discriminant);
        let a_recip = 1.0 / a;
        let t0 = (-half_b - ds) * a_recip;
        let t1 = (-half_b + ds) * a_recip;

        // Calculates positions of the intersections
        let pos0 = ray.position + t0 * ray.direction;
        let pos1 = ray.position + t1 * ray.direction;

        // Calculates normals at the points of intersection
        let radius_recip = 1.0 / self.radius;
        let normal0 = (pos0 - self.centre) * radius_recip;
        let mut normal1 = (pos1 - self.centre) * radius_recip;

        // Negates normal facing away from the ray
        normal1 = normal1.neg();

        // Creates hit objects, including texture coordinates for the nearer hit
        let mut hit0 = Hit::new(
            t0,
            true,
            pos0,
            normal0,
            &self.material,
            Some(Self::get_tex_coords(normal0)),
        );
        let mut hit1 = Hit::new(t1, false, pos1, normal1, &self.material, None);

        // Transforms the hits back into world space
        hit0.transform(&self.transform);
        hit1.transform(&self.transform);
        vec![hit0, hit1]
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

/// Implements loading a `Sphere` from a YAML file.
impl FromYaml for Sphere {
    fn from_yaml(yaml: &Yaml) -> Result<Sphere, YamlPropertyError> {
        // Parses properties for the sphere
        let position = parse_struct(yaml, "position")?;
        let radius = parse_float(yaml, "radius")?;
        let material = parse_string(yaml, "material")?;

        // Creates the sphere instance
        let mut sphere = Sphere::new(position, radius, &material);

        // Applies any present transforms to the sphere
        let transform = parse_transforms(yaml)?;
        sphere.apply_transform(transform);

        Ok(sphere)
    }
}
