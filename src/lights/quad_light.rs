use crate::core::Ray;
use crate::drawing::Colour;
use crate::lights::LightTrait;
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, Vector, Vertex};
use yaml_rust::Yaml;

#[derive(Debug)]
/// A diffuse quadrilateral area light, defined in the same way as `Quad` objects.
pub struct QuadLight {
    point: Vertex, // Starting point of the quadrilateral
    edge1: Vector, // First side of the quadrilateral
    edge2: Vector, // Second side of the quadrilateral

    normal: Vector, // Normal of the quadrilateral
    area: f64,      // Area of the quadrilateral

    intensity: Colour, // Intensity of the light
}

impl QuadLight {
    /// Creates a new `QuadLight` with the given `Quad` specification and intensity.
    pub fn new(point: Vertex, edge1: Vector, edge2: Vector, intensity: Colour) -> QuadLight {
        // Calculates the cross product of the two edges
        let cross = Vector::cross(&edge1, &edge2);

        // Uses the cross product to get the normal and area of the quadrilateral
        let normal = cross.unit();
        let area = cross.length();

        // Constructs the QuadLight instance
        QuadLight {
            point,
            edge1,
            edge2,
            normal,
            area,
            intensity,
        }
    }
}

impl LightTrait for QuadLight {
    fn sample_point(&self) -> Vertex {
        // Generates random uv coordinates
        let u = random_float();
        let v = random_float();

        // Returns a new point on the surface of the quadrilateral
        self.point + u * self.edge1 + v * self.edge2
    }

    fn sample_photon_ray(&self) -> Ray {
        // Samples a random point and direction
        let p = self.sample_point();
        let d = Vector::sample_diffuse_vector(&self.normal);

        // Creates and returns the photon ray
        Ray::offset(p, d)
    }

    fn get_intensity(&self) -> Colour {
        self.intensity
    }

    fn get_intensity_at_point(&self, _point: &Vertex, light_ray: &Vector, distance: f64) -> Colour {
        // Scales the intensity with quadratic distance attenuation, and by the cosine of the light's angle
        self.intensity * Vector::dot(light_ray, &self.normal).max(0.0) / (distance * distance)
    }

    fn get_area(&self) -> f64 {
        self.area
    }
}

/// Implements loading a `QuadLight` from a YAML file.
impl FromYaml for QuadLight {
    fn from_yaml(yaml: &Yaml) -> Result<QuadLight, YamlPropertyError> {
        // Parses properties for the light
        let point = parse_struct(yaml, "point")?;
        let edge1 = parse_struct(yaml, "edge1")?;
        let edge2 = parse_struct(yaml, "edge2")?;
        let intensity = parse_struct(yaml, "intensity")?;

        Ok(QuadLight::new(point, edge1, edge2, intensity))
    }
}
