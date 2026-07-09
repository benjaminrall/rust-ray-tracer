use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::Light;
use crate::materials::MaterialTrait;
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use crate::utils::ScatterType;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Material to represent emissive objects.
/// Works by returning a single flat colour value, which should be equal to the intensity of the light source.
pub struct EmissiveMaterial {
    intensity: Colour,
}

impl EmissiveMaterial {
    /// Creates a new `EmissiveMaterial` with the given intensity.
    pub fn new(intensity: Colour) -> EmissiveMaterial {
        EmissiveMaterial { intensity }
    }
}

impl MaterialTrait for EmissiveMaterial {
    fn compute_once(
        &self,
        _incident: &Ray,
        hit: &Hit,
        _recurse: i32,
        _scene: &Scene,
        _direct: bool,
    ) -> Colour {
        // Returns the intensity only if the emitting side of the object was hit
        if hit.entering {
            self.intensity
        } else {
            Colour::black()
        }
    }

    fn compute_per_light(
        &self,
        _incident: &Ray,
        _hit: &Hit,
        _light: &Light,
        _scene: &Scene,
        _direct: bool,
    ) -> Colour {
        Colour::black()
    }

    fn compute_per_photon(&self, _incident: &Ray, _hit: &Hit, _photon: &Photon) -> Colour {
        Colour::black()
    }

    fn scatter_photon(
        &self,
        _photon_ray: &Ray,
        _hit: &Hit,
        _power: &mut Colour,
    ) -> Option<(Ray, ScatterType)> {
        None
    }
}

/// Implements loading an `EmissiveMaterial` from a YAML file.
impl FromYaml for EmissiveMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<EmissiveMaterial, YamlPropertyError> {
        // Parses properties for the material
        let intensity = parse_struct(yaml, "intensity")?;

        // Returns the new material instance
        Ok(EmissiveMaterial::new(intensity))
    }
}
