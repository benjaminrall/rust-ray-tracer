use crate::core::{Hit, Ray};
use crate::drawing::Colour;
use crate::lights::LightTrait;
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{Vector, Vertex};
use std::f64::consts::PI;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Simple point light that emits light in all directions from a single position.
pub struct PointLight {
    position: Vertex,
    intensity: Colour,
}

impl PointLight {
    /// Creates a new `PointLight` instance with a specified position and intensity.
    pub fn new(position: Vertex, intensity: Colour) -> PointLight {
        PointLight {
            position,
            intensity,
        }
    }
}

impl LightTrait for PointLight {
    fn sample_point(&self) -> Vertex {
        // Point light exists only at a single point
        self.position
    }

    fn sample_n_points(&self, _hit: &Hit, _n: usize) -> Vec<Vertex> {
        // Overrides sampling n points with sampling one, since the result is the same
        vec![self.position]
    }

    fn sample_photon_ray(&self) -> Ray {
        // Samples a random direction in a sphere around the point light
        let d = Vector::random_unit_vector();
        Ray::new(self.position, d)
    }

    fn get_intensity(&self) -> Colour {
        self.intensity
    }

    fn get_intensity_at_point(
        &self,
        _point: &Vertex,
        _light_ray: &Vector,
        distance: f64,
    ) -> Colour {
        // Applies quadratic distance attenuation to the point light's intensity
        self.intensity / (distance * distance)
    }

    fn get_area(&self) -> f64 {
        // Treats point lights as having an area of 1 for calculations
        1.0
    }
}

/// Implements loading a `PointLight` from a YAML file.
impl FromYaml for PointLight {
    fn from_yaml(yaml: &Yaml) -> Result<PointLight, YamlPropertyError> {
        let position = parse_struct(yaml, "position")?;
        let intensity = parse_struct(yaml, "intensity")?;

        Ok(PointLight::new(position, intensity))
    }
}
