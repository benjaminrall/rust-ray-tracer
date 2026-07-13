use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::Light;
use crate::materials::MaterialTrait;
use crate::textures::{Texture, TextureTrait};
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::ScatterType;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Material to represent emissive volumes.
/// Works by returning a flat colour (optionally from a texture),
/// which should be equal to the intensity of the light source.
pub struct EmissiveVolumeMaterial {
    intensity: Texture,
    density: f64,
}

impl EmissiveVolumeMaterial {
    /// Creates a new `EmissiveVolumeMaterial` with the given intensity and simulated density.
    pub fn new(intensity: Texture, density: f64) -> EmissiveVolumeMaterial {
        EmissiveVolumeMaterial { intensity, density }
    }
}

impl MaterialTrait for EmissiveVolumeMaterial {
    fn compute_once(
        &self,
        _incident: &Ray,
        hit: &Hit,
        _recurse: i32,
        _scene: &Scene,
        _direct: bool,
    ) -> Colour {
        self.intensity.get_colour_at(hit) * self.density
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

    fn is_diffuse(&self) -> bool {
        false
    }
}

/// Implements loading a `EmissiveVolumeMaterial` from a YAML file.
impl FromYaml for EmissiveVolumeMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<EmissiveVolumeMaterial, YamlPropertyError> {
        // Parses properties for the material
        let intensity = parse_struct(yaml, "intensity")?;
        let density = parse_float(yaml, "density")?;

        Ok(EmissiveVolumeMaterial::new(intensity, density))
    }
}
