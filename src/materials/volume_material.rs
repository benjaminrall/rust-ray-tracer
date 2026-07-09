use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::{Light, LightTrait};
use crate::materials::{LambertianMaterial, MaterialTrait};
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, ScatterType, Vector};
use crate::{DIRECT_SAMPLES, EPSILON};
use std::f64::consts::PI;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Material to be used by a scattering volume.
pub struct VolumeMaterial {
    colour: Colour,                     // The colour component of the scattering volume
    scatter_coefficients_recip: Colour, // Reciprocal of the scatter coefficients used for indirect lighting
}

impl VolumeMaterial {
    /// Creates a new `VolumeMaterial` instance with the given colour and density.
    pub fn new(colour: Colour, density: f64) -> VolumeMaterial {
        // Calculates the reciprocal scatter coefficients from the volume's density and colour
        let scatter_coefficients = colour * density;
        let scatter_coefficients_recip = Colour::new(
            1.0 / scatter_coefficients.r,
            1.0 / scatter_coefficients.g,
            1.0 / scatter_coefficients.b,
        );

        VolumeMaterial {
            colour,
            scatter_coefficients_recip,
        }
    }
}

impl MaterialTrait for VolumeMaterial {
    fn compute_once(
        &self,
        incident: &Ray,
        hit: &Hit,
        _recurse: i32,
        scene: &Scene,
        _direct: bool,
    ) -> Colour {
        // Computes and returns the volume's indirect lighting
        self.scatter_coefficients_recip
            * scene.volume_radiance_estimate(hit, |photon| {
                self.compute_per_photon(incident, hit, photon)
            })
    }

    fn compute_per_light(
        &self,
        _incident: &Ray,
        hit: &Hit,
        light: &Light,
        scene: &Scene,
        direct: bool,
    ) -> Colour {
        // Gets a list of sample points on the light
        let n_samples = if direct { DIRECT_SAMPLES } else { 1 };
        let samples = light.sample_n_points(hit, n_samples);
        let total_samples = samples.len();

        // For each sample point
        let mut scattered = Colour::black();
        for sample in samples {
            // Calculates the direction and distance to the light sample
            let to_light = sample - hit.position;
            let direction = to_light.unit();
            let distance = to_light.length();

            // Creates a ray between the hit position and the sampled light point
            let light_ray = Ray::offset(hit.position, direction);

            // Checks if the ray is in shadow
            if scene.shadow_trace(&light_ray, distance - EPSILON) {
                continue;
            }

            // Gets the intensity at the hit position using the distance from the light point
            let intensity = light.get_intensity_at_point(&sample, &-direction, distance);

            // Adds scattered reflection from the sample point to the total
            scattered += intensity;
        }

        // Averages the scattered contribution over all samples and the area of the light
        // Uses a uniform probability density function for the sphere surrounding the hit point
        if total_samples > 0 {
            scattered *= self.colour * light.get_area() / (total_samples as f64 * 4.0 * PI);
        }

        scattered
    }

    fn compute_per_photon(&self, _incident: &Ray, _hit: &Hit, photon: &Photon) -> Colour {
        self.colour * photon.power
    }

    fn scatter_photon(
        &self,
        _photon_ray: &Ray,
        hit: &Hit,
        power: &mut Colour,
    ) -> Option<(Ray, ScatterType)> {
        // Calculates the probability of the photon scattering
        let ps = (self.colour * *power).max() / power.max();

        // Performs Russian Roulette using this probability
        let eta = random_float();
        if eta < ps {
            // Photon is reflected - gets a random scattered direction in a sphere
            let reflected_dir = Vector::random_unit_vector();

            // Adjusts the power of the reflected photon by its probability of survival
            *power *= self.colour / ps;

            // Returns the scattered ray
            Some((
                Ray::offset(hit.position, reflected_dir),
                ScatterType::Volume,
            ))
        } else {
            // Photon is absorbed
            None
        }
    }
}

/// Implements loading a `VolumeMaterial` from a YAML file.
impl FromYaml for VolumeMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<VolumeMaterial, YamlPropertyError> {
        // Parses properties for the material
        let colour = parse_struct(yaml, "colour")?;
        let density = parse_float(yaml, "density")?;

        Ok(VolumeMaterial::new(colour, density))
    }
}
