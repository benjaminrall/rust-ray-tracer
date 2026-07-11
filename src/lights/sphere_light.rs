use crate::core::Ray;
use crate::drawing::Colour;
use crate::lights::LightTrait;
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{Vector, Vertex};
use std::f64::consts::PI;
use yaml_rust::Yaml;

#[derive(Debug)]
/// A diffuse sphere area light, defined in the same way as `Sphere` objects.
pub struct SphereLight {
    position: Vertex, // Position of the sphere
    radius: f64,      // Radius of the sphere

    area: f64,        // Surface area of the sphere

    intensity: Colour, // Intensity of the light
}

impl SphereLight {
    /// Creates a new `SphereLight` with the given `Sphere` specification and intensity.
    pub fn new(position: Vertex, radius: f64, intensity: Colour) -> SphereLight {
        // Calculates the surface area of the sphere
        let area = 4.0 * PI * radius * radius;

        // Constructs the QuadLight instance
        SphereLight {
            position,
            radius,
            area,
            intensity,
        }
    }

    /// Returns the normal of a point on the sphere's surface.
    fn normal(&self, point: &Vertex) -> Vector {
        (*point - self.position).unit()
    }
}

impl LightTrait for SphereLight {
    fn sample_point(&self) -> Vertex {
        // Samples a random point on the surface of the sphere
        let direction = Vector::random_unit_vector();
        self.position + (direction * self.radius)
    }

    fn sample_photon_ray(&self) -> Ray {
        // Samples a random point and direction
        let p = self.sample_point();
        let d = Vector::sample_diffuse_vector(&self.normal(&p));

        // Creates and returns the photon ray
        Ray::offset(p, d)
    }

    fn get_intensity(&self) -> Colour {
        self.intensity
    }

    fn get_intensity_at_point(&self, point: &Vertex, light_ray: &Vector, distance: f64) -> Colour {
        // Applies cosine fall-off and quadratic distance attenuation to the intensity
        self.intensity * Vector::dot(light_ray, &self.normal(point)).max(0.0)
            / (distance * distance)
    }

    fn get_area(&self) -> f64 {
        self.area
    }
}

/// Implements loading a `SphereLight` from a YAML file.
impl FromYaml for SphereLight {
    fn from_yaml(yaml: &Yaml) -> Result<SphereLight, YamlPropertyError> {
        // Parses properties for the light
        let position = parse_struct(yaml, "position")?;
        let radius = parse_float(yaml, "radius")?;
        let intensity = parse_struct(yaml, "intensity")?;

        Ok(SphereLight::new(position, radius, intensity))
    }
}
