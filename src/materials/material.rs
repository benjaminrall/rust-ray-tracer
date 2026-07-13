use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::Light;
use crate::materials::{
    EmissiveMaterial, EmissiveVolumeMaterial, GlobalMaterial, LambertianMaterial, PhongMaterial,
    VolumeMaterial,
};
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use crate::utils::ScatterType;
use enum_dispatch::enum_dispatch;
use yaml_rust::Yaml;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent all materials.
pub enum Material {
    LambertianMaterial,
    EmissiveMaterial,
    GlobalMaterial,
    PhongMaterial,
    VolumeMaterial,
    EmissiveVolumeMaterial,
}

#[enum_dispatch(Material)]
/// Trait which must be implemented by all `Material` structs.
pub trait MaterialTrait {
    /// Computes the component of the material's colour per intersection.
    ///
    /// Equivalent to ambient, reflected, transmitted, caustic, and indirect lighting contributions.
    ///
    /// # Arguments
    ///
    /// * `incident`: Direction of the incident ray.
    /// * `hit`: Record of where the object was hit by the incident ray.
    /// * `recurse`: Number of times to recurse.
    /// * `scene`: Reference to the current scene for recursive ray tracing.
    /// * `direct`: Whether the material was hit directly from the camera.
    fn compute_once(
        &self,
        incident: &Ray,
        hit: &Hit,
        recurse: i32,
        scene: &Scene,
        direct: bool,
    ) -> Colour;

    /// Computes the component of the material's colour per light source.
    ///
    /// Equivalent to the direct lighting contribution.
    ///
    /// # Arguments
    ///
    /// * `incident`: Direction of the incident ray.
    /// * `hit`: Record of where the object was hit by the incident ray.
    /// * `light`: Reference to the current light source being evaluated.
    /// * `scene`: Reference to the current scene for shadow tracing.
    /// * `direct`: Whether the hit is direct from the camera.
    fn compute_per_light(
        &self,
        incident: &Ray,
        hit: &Hit,
        light: &Light,
        scene: &Scene,
        direct: bool,
    ) -> Colour;

    /// Computes the contribution of a photon to the surface's colour.
    ///
    /// # Arguments
    ///
    /// * `incident`: Direction of the incident ray.
    /// * `hit`: Hit record at which to calculate the photon's contribution.
    /// * `photon`: Photon to calculate the colour contribution from.
    fn compute_per_photon(&self, incident: &Ray, hit: &Hit, photon: &Photon) -> Colour;

    /// Scatters an incident photon ray based on the material's properties,
    /// using Russian roulette to decide the type of interaction with the surface.
    ///
    /// # Arguments
    ///
    /// * `photon_ray`: Incident ray of the photon.
    /// * `hit`: Information about the photon's hit.
    /// * `power`: The power of the photon, modified by reference if a new photon ray is returned.
    ///
    /// returns: `None` if the photon is absorbed by the material. Otherwise, returns both a new `Ray`
    /// representing the scattered direction of the photon after interaction, and an enum indicating
    /// the type of scattering interaction that occurred.
    fn scatter_photon(
        &self,
        photon_ray: &Ray,
        hit: &Hit,
        power: &mut Colour,
    ) -> Option<(Ray, ScatterType)>;

    /// Returns the material's primary scatter type.
    fn is_diffuse(&self) -> bool;
}

/// Implements loading `Material` structs from a YAML file.
impl FromYaml for Material {
    fn from_yaml(yaml: &Yaml) -> Result<Material, YamlPropertyError> {
        // Parses material type as a String
        let material_type = parse_string(yaml, "type")?;

        // Matches the type to its respective object
        match material_type.as_str() {
            "LambertianMaterial" => Ok(LambertianMaterial::from_yaml(yaml)?.into()),
            "PhongMaterial" => Ok(PhongMaterial::from_yaml(yaml)?.into()),
            "EmissiveMaterial" => Ok(EmissiveMaterial::from_yaml(yaml)?.into()),
            "GlobalMaterial" => Ok(GlobalMaterial::from_yaml(yaml)?.into()),
            "VolumeMaterial" => Ok(VolumeMaterial::from_yaml(yaml)?.into()),
            "EmissiveVolumeMaterial" => Ok(EmissiveVolumeMaterial::from_yaml(yaml)?.into()),
            _ => Err(YamlPropertyError::invalid("type")),
        }
    }
}
