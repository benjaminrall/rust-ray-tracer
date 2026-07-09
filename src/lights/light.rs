use crate::core::{Hit, Ray};
use crate::drawing::Colour;
use crate::lights::{DirectionalQuadLight, PointLight, QuadLight, SphereLight};
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use crate::utils::{Vector, Vertex};
use enum_dispatch::enum_dispatch;
use yaml_rust::Yaml;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent all light sources.
pub enum Light {
    PointLight,
    QuadLight,
    SphereLight,
    DirectionalQuadLight,
}

#[enum_dispatch(Light)]
/// Trait which must be implemented by all `Light` structs.
pub trait LightTrait {
    /// Samples a random point from the surface of the light.
    fn sample_point(&self) -> Vertex;

    /// Samples `n` random points from the surface of the light.
    fn sample_n_points(&self, _hit: &Hit, n: usize) -> Vec<Vertex> {
        (0..n).map(|_| self.sample_point()).collect()
    }

    /// Samples a random photon ray from the light into the scene.
    fn sample_photon_ray(&self) -> Ray;

    /// Returns the base intensity of the light.
    fn get_intensity(&self) -> Colour;

    /// Gets the intensity of the light at a point on a surface,
    /// given its direction and distance from the light source.
    ///
    /// # Arguments
    ///
    /// * `point`: The point on the surface of the light.
    /// * `light_ray`: Unit vector from the point on the light to the surface.
    /// * `distance`: Distance between the point and the surface.
    fn get_intensity_at_point(&self, point: &Vertex, light_ray: &Vector, distance: f64) -> Colour;

    /// Returns the area of the light.
    fn get_area(&self) -> f64;
}

/// Implements loading `Light` structs from a YAML file.
impl FromYaml for Light {
    fn from_yaml(yaml: &Yaml) -> Result<Light, YamlPropertyError> {
        // Parses light type as a String
        let light_type = parse_string(yaml, "type")?;

        // Matches the type to its respective object
        match light_type.as_str() {
            "PointLight" => Ok(PointLight::from_yaml(yaml)?.into()),
            "QuadLight" => Ok(QuadLight::from_yaml(yaml)?.into()),
            "SphereLight" => Ok(SphereLight::from_yaml(yaml)?.into()),
            "DirectionalQuadLight" => Ok(DirectionalQuadLight::from_yaml(yaml)?.into()),
            _ => Err(YamlPropertyError::invalid("type")),
        }
    }
}
