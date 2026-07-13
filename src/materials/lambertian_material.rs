use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::{Light, LightTrait};
use crate::materials::MaterialTrait;
use crate::textures::{Texture, TextureTrait};
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, ScatterType, Vector};
use crate::{DIRECT_SAMPLES, EPSILON, INDIRECT_SAMPLES};
use std::f64::consts::PI;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Struct to represent a purely Lambertian diffuse material.
pub struct LambertianMaterial {
    albedo: Texture, // Surface colour of the material as a texture
}

impl LambertianMaterial {
    /// Creates a new `LambertianMaterial` with the given diffuse surface colour.
    pub fn new(albedo: Texture) -> LambertianMaterial {
        LambertianMaterial { albedo }
    }
}

impl MaterialTrait for LambertianMaterial {
    fn compute_once(
        &self,
        incident: &Ray,
        hit: &Hit,
        _recurse: i32,
        scene: &Scene,
        direct: bool,
    ) -> Colour {
        // Computes indirect component of the colour
        let mut estimate = Colour::black();
        let monte_carlo_rays = if direct { INDIRECT_SAMPLES } else { 1 };
        for _ in 0..monte_carlo_rays {
            let d = Vector::sample_diffuse_vector(&hit.normal);
            let ray = Ray::offset(hit.position, d);
            estimate += scene.indirect_raytrace(ray);
        }
        let indirect = (estimate / monte_carlo_rays as f64) * self.albedo.get_colour_at(hit);

        // Computes caustic component of the colour
        let caustic = scene.caustic_radiance_estimate(hit, |photon| {
            self.compute_per_photon(incident, hit, photon)
        });

        // Returns total per intersection colour contribution
        caustic + indirect
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
        let mut diffuse = Colour::black();
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

            // Adds diffuse reflection from the sample point to the total
            diffuse += intensity * f64::max(Vector::dot(&hit.normal, &direction), 0.0);
        }

        // Averages the diffuse contribution over all samples and the area of the light
        // Additionally scaled by the probability density function of Lambertian diffuse (1 / pi)
        if total_samples > 0 {
            diffuse *=
                self.albedo.get_colour_at(hit) * light.get_area() / (total_samples as f64 * PI);
        }

        diffuse
    }

    fn compute_per_photon(&self, _incident: &Ray, hit: &Hit, photon: &Photon) -> Colour {
        // Gets the photon's colour contribution using its power and the Lambertian diffuse BRDF
        self.albedo.get_colour_at(hit)
            * Vector::dot(&hit.normal, &photon.direction).max(0.0)
            * photon.power
    }

    fn scatter_photon(
        &self,
        _photon_ray: &Ray,
        hit: &Hit,
        power: &mut Colour,
    ) -> Option<(Ray, ScatterType)> {
        // Calculates the probability of there being a diffuse reflection
        let albedo = self.albedo.get_colour_at(hit);
        let pd = (albedo * *power).max() / power.max();

        // Performs Russian Roulette using the diffuse probability
        let eta = random_float();
        if eta < pd {
            // Photon is reflected - gets a random diffuse reflection direction
            let reflected_dir = Vector::sample_diffuse_vector(&hit.normal);

            // Adjusts the power of the reflected photon by its probability of survival
            *power *= albedo / pd;

            // Returns the scattered ray
            Some((
                Ray::offset(hit.position, reflected_dir),
                ScatterType::Diffuse,
            ))
        } else {
            // Photon is absorbed
            None
        }
    }

    fn is_diffuse(&self) -> bool {
        true
    }
}

/// Implements loading a `LambertianMaterial` from a YAML file.
impl FromYaml for LambertianMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<LambertianMaterial, YamlPropertyError> {
        // Parses properties for the material
        let albedo = parse_struct(yaml, "albedo")?;

        Ok(LambertianMaterial::new(albedo))
    }
}
