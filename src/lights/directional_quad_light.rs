use crate::core::{Hit, Ray};
use crate::drawing::Colour;
use crate::lights::LightTrait;
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, Vector, Vertex};
use crate::EPSILON;
use yaml_rust::Yaml;

#[derive(Debug)]
/// A directional quadrilateral light, defined in the same way as `Quad` objects.
/// Used to simulate infinitely far light sources passing through a finite quadrilateral area.
pub struct DirectionalQuadLight {
    point: Vertex, // Starting point of the quadrilateral
    edge1: Vector, // First side of the quadrilateral
    edge2: Vector, // Second side of the quadrilateral
    
    intensity: Colour, // Intensity of the light
    direction: Vector, // Direction of the light
}

impl DirectionalQuadLight {
    // Simulated distance for sampling a point from the light for shadow calculations.
    // Should ideally be infinite, so increase for larger scenes.
    pub const SIMULATED_DISTANCE: f64 = 100.0;

    /// Creates a new `DirectionalQuadLight` with the given `Quad` specification, intensity, and direction.
    pub fn new(
        point: Vertex,
        edge1: Vector,
        edge2: Vector,
        intensity: Colour,
        direction: Vector,
    ) -> DirectionalQuadLight {
        DirectionalQuadLight {
            point,
            edge1,
            edge2,
            intensity,
            direction: direction.unit(),
        }
    }
}

impl LightTrait for DirectionalQuadLight {
    fn sample_point(&self) -> Vertex {
        // Generates random uv coordinates
        let u = random_float();
        let v = random_float();

        // Returns a new point on the surface of the quadrilateral
        self.point + u * self.edge1 + v * self.edge2
    }

    fn sample_n_points(&self, hit: &Hit, _n: usize) -> Vec<Vertex> {
        // Overrides sampling `n` points to instead sample a singular point a simulated infinite distance away
        vec![hit.position + hit.normal * EPSILON - self.direction * Self::SIMULATED_DISTANCE]
    }

    fn sample_photon_ray(&self) -> Ray {
        // Samples a random point and returns a ray with that origin in the light's direction
        let p = self.sample_point();
        Ray::offset(p, self.direction)
    }

    fn get_intensity(&self) -> Colour {
        self.intensity
    }

    fn get_intensity_at_point(
        &self,
        _point: &Vertex,
        _light_ray: &Vector,
        _distance: f64,
    ) -> Colour {
        self.intensity
    }

    fn get_area(&self) -> f64 {
        1.0
    }
}

/// Implements loading a `DirectionalQuadLight` from a YAML file.
impl FromYaml for DirectionalQuadLight {
    fn from_yaml(yaml: &Yaml) -> Result<DirectionalQuadLight, YamlPropertyError> {
        // Parses properties for the light
        let point = parse_struct(yaml, "point")?;
        let edge1 = parse_struct(yaml, "edge1")?;
        let edge2 = parse_struct(yaml, "edge2")?;
        let intensity = parse_struct(yaml, "intensity")?;
        let direction = parse_struct(yaml, "direction")?;

        Ok(DirectionalQuadLight::new(
            point, edge1, edge2, intensity, direction,
        ))
    }
}
