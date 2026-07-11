use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::{Light, LightTrait};
use crate::materials::MaterialTrait;
use crate::textures::{Texture, TextureTrait};
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, ScatterType, Vector};
use crate::{AIR_IOR, DIRECT_SAMPLES, EPSILON, INDIRECT_SAMPLES, SPECULAR_SAMPLES};
use std::f64::consts::FRAC_1_PI;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Material that uses the Phong reflection model to calculate ambient, diffuse, and specular
/// colour contributions.
///
/// Influenced by the modified physically-based Phong model proposed by Lafortune.
/// https://www.researchgate.net/publication/2361953_Using_the_Modied_Phong_Reflectance_Model_for_Physically_Based_Rendering
pub struct PhongMaterial {
    ambient: Texture,  // Ambient component of the material
    diffuse: Texture,  // Diffuse component of the material
    specular: Texture, // Specular component of the material

    alpha: f64, // Phong shininess constant
    ior: f64,   // Index of refraction of the material
}

impl PhongMaterial {
    /// Creates a new `PhongMaterial` instance.
    ///
    /// # Arguments
    ///
    /// * `ambient`: Ambient reflection component.
    /// * `diffuse`: Diffuse reflection component.
    /// * `specular`: Specular reflection component.
    /// * `alpha`: Shininess constant.
    /// * `ior`: Index of refraction of the material.
    pub fn new(
        ambient: Texture,
        diffuse: Texture,
        specular: Texture,
        alpha: f64,
        ior: f64,
    ) -> PhongMaterial {
        PhongMaterial {
            ambient,
            diffuse,
            specular,
            alpha,
            ior,
        }
    }

    /// Calculates the relative refractive indices and refractive ratio of the materials
    /// either side of a hit. Assumes that the other material is air for simplicity.
    fn calculate_refractive_indices(&self, hit: &Hit) -> (f64, f64, f64) {
        // Orders refractive indices based on whether the hit is entering or exiting an object
        let (n1, n2) = if hit.entering {
            (AIR_IOR, self.ior)
        } else {
            (self.ior, AIR_IOR)
        };

        // Calculates ratio between in and out refractive indices
        let nr = n1 / n2;

        (n1, n2, nr)
    }

    /// Uses Schlick's approximation to compute the Fresnel reflectance value.
    ///
    /// # Arguments
    ///
    /// * `n1`: Refractive index of the first medium (being exited).
    /// * `n2`: Refractive index of the second medium (being entered).
    /// * `cos_i`: Cosine of the angle of incidence.
    /// * `sin_t_sqr`: Squared sine value of the angle of refraction.
    /// * `cos_t`: Cosine of the angle of refraction.
    fn schlick(n1: f64, n2: f64, cos_i: f64, sin_t_sqr: f64, cos_t: f64) -> f64 {
        // Calculates reflection coefficient for rays incoming parallel to the normal
        let r0 = f64::powi((n1 - n2) / (n1 + n2), 2);

        // Checks for total internal reflection
        if n1 > n2 && sin_t_sqr > 1.0 {
            return 1.0;
        }

        // Determines which angle to use for calculating reflectance
        let x = 1.0 - if n1 > n2 { cos_t } else { cos_i };
        r0 + (1.0 - r0) * x.powi(5)
    }
}

impl MaterialTrait for PhongMaterial {
    fn compute_once(
        &self,
        incident: &Ray,
        hit: &Hit,
        recurse: i32,
        scene: &Scene,
        direct: bool,
    ) -> Colour {
        // Does not recurse if the recursion limit has been met
        if recurse <= 0 {
            return Colour::black();
        }

        // Gets diffuse and specular colours at the hit
        let mut kd = self.diffuse.get_colour_at(hit);
        let mut ks = self.specular.get_colour_at(hit);

        // Normalises them to conserve energy
        let norm = (kd + ks).max();
        if norm > 1.0 {
            kd /= norm;
            ks /= norm;
        }

        // Gets refractive indices for the hit
        let (n1, n2, nr) = self.calculate_refractive_indices(hit);

        // Calculates angle values for the hit from Snell's law
        let cos_i = -Vector::dot(&hit.normal, &incident.direction);
        let sin_t_sqr = nr * nr * (1.0 - cos_i * cos_i);
        let cos_t = (1. - sin_t_sqr).sqrt();

        // Computes Fresnel reflectance using Schlick's approximation
        let reflectance = Self::schlick(n1, n2, cos_i, sin_t_sqr, cos_t);

        // Shoots reflected specular rays into the scene and calculates their colour contribution
        let reflect_dir = incident.direction + 2.0 * cos_i * hit.normal;
        let mut colour = Colour::black();
        let monte_carlo_rays = if direct { SPECULAR_SAMPLES } else { 1 };
        for _ in 0..monte_carlo_rays {
            let sample_dir = Vector::sample_specular_vector(&reflect_dir, cos_i, self.alpha);

            // Traces the monte carlo ray to get its colour contribution
            let ray = Ray::offset(hit.position, sample_dir);
            colour += scene.raytrace(&ray, recurse - 1, false);
        }
        let reflection = colour / monte_carlo_rays as f64;

        // Computes indirect component of the colour
        let mut estimate = Colour::black();
        let monte_carlo_rays = if direct { INDIRECT_SAMPLES } else { 1 };
        for _ in 0..monte_carlo_rays {
            let d = Vector::sample_diffuse_vector(&hit.normal);
            let ray = Ray::offset(hit.position, d);
            estimate += scene.indirect_raytrace(ray);
        }
        let indirect = estimate / monte_carlo_rays as f64;

        // Computes caustic component of the colour
        let caustic = scene.caustic_radiance_estimate(hit, |photon| {
            self.compute_per_photon(incident, hit, photon)
        });

        // Returns total colour contribution
        self.ambient.get_colour_at(hit)
            + caustic
            + (ks + reflectance * kd) * reflection
            + (1.0 - reflectance) * indirect * kd
    }

    fn compute_per_light(
        &self,
        _incident: &Ray,
        hit: &Hit,
        light: &Light,
        scene: &Scene,
        direct: bool,
    ) -> Colour {
        // Gets diffuse and specular colours at the hit
        let mut kd = self.diffuse.get_colour_at(hit);
        let mut ks = self.specular.get_colour_at(hit);

        // Normalises them to conserve energy
        let norm = (kd + ks).max();
        if norm > 1.0 {
            kd /= norm;
            ks /= norm;
        }

        // Gets a list of sample points on the light
        let n_samples = if direct { DIRECT_SAMPLES } else { 1 };
        let samples = light.sample_n_points(hit, n_samples);
        let total_samples = samples.len();

        // For each sample point
        let mut diffuse = Colour::black();
        let mut specular = Colour::black();
        for sample in samples {
            // Calculates the direction and distance to the light sample
            let to_light = sample - hit.position;
            let direction = to_light.unit();
            let distance = to_light.length();

            // Creates a ray between the hit position and the sampled light point
            let light_ray = Ray::offset(hit.position + EPSILON * hit.normal, direction);

            // Checks if the ray is in shadow
            if scene.shadow_trace(&light_ray, distance - EPSILON) {
                continue;
            }

            // Gets the intensity at the hit position using the distance from the light point
            let intensity = light.get_intensity_at_point(&sample, &-direction, distance);

            // Adds diffuse reflection from the sample point to the total
            diffuse += intensity * Vector::dot(&hit.normal, &direction).max(0.0);

            // Adds specular reflection from the sample point to the total
            let reflection = Vector::reflection(&hit.normal, &direction);
            let view_angle = Vector::dot(&_incident.direction, &reflection).max(0.0);
            specular += intensity * view_angle.powf(self.alpha)
        }

        // Averages the diffuse and specular contributions over all samples and the area of the light
        // Uses probability density functions given by Lafortune for a physically based Phong model.
        if total_samples > 0 {
            let multiplier = light.get_area() / (total_samples as f64);
            diffuse *= kd * multiplier * FRAC_1_PI;
            specular *= ks * multiplier * (self.alpha + 1.0) * FRAC_1_PI * 0.5;
        }

        diffuse + specular
    }

    fn compute_per_photon(&self, incident: &Ray, hit: &Hit, photon: &Photon) -> Colour {
        // View angle calculation for specular component
        let reflection = Vector::reflection(&hit.normal, &photon.direction);
        let view_angle = Vector::dot(&incident.direction, &reflection).max(0.0);

        // Total photon colour contribution
        (self.diffuse.get_colour_at(hit) * Vector::dot(&hit.normal, &photon.direction).max(0.0)
            + self.specular.get_colour_at(hit) * view_angle.powf(self.alpha))
            * photon.power
    }

    fn scatter_photon(
        &self,
        photon_ray: &Ray,
        hit: &Hit,
        power: &mut Colour,
    ) -> Option<(Ray, ScatterType)> {
        // Gets refractive indices for the hit
        let (n1, n2, nr) = self.calculate_refractive_indices(hit);

        // Calculates angle values for the hit from Snell's law
        let cos_i = -Vector::dot(&hit.normal, &photon_ray.direction);
        let sin_t_sqr = nr * nr * (1.0 - cos_i * cos_i);
        let cos_t = (1. - sin_t_sqr).sqrt();

        // Computes Fresnel reflectance using Schlick's approximation
        let reflectance = Self::schlick(n1, n2, cos_i, sin_t_sqr, cos_t);

        // Gets diffuse and specular colours at the hit
        let mut kd = self.diffuse.get_colour_at(hit);
        let mut ks = self.specular.get_colour_at(hit);

        // Normalises them to conserve energy
        let norm = (kd + ks).max();
        if norm > 1.0 {
            kd /= norm;
            ks /= norm;
        }

        // Computes probabilities of diffuse and specular reflection
        let diffuse_max = kd.max();
        let ps = ks.max() + reflectance * diffuse_max;
        let pd = (1.0 - reflectance) * diffuse_max;

        // Selects whether to perform diffuse or specular reflection
        let eta = random_float();
        if eta < pd {
            // Diffuse reflection
            let reflect_dir = Vector::sample_diffuse_vector(&hit.normal);
            *power *= kd / pd;
            Some((Ray::offset(hit.position, reflect_dir), ScatterType::Diffuse))
        } else if eta < pd + ps {
            // Specular reflection
            let cos_i = -Vector::dot(&hit.normal, &photon_ray.direction);
            let ideal_dir = Vector::reflection(&hit.normal, &photon_ray.direction);
            let reflect_dir = Vector::sample_specular_vector(&ideal_dir, cos_i, self.alpha);
            *power *= ks / ps;
            Some((
                Ray::offset(hit.position, reflect_dir),
                ScatterType::Specular,
            ))
        } else {
            // Photon is absorbed, so return no scattered ray
            None
        }
    }
}

/// Implements loading a `PhongMaterial` from a YAML file.
impl FromYaml for PhongMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<PhongMaterial, YamlPropertyError> {
        // Reads in the material's coefficients
        let ambient = parse_struct(yaml, "ambient").unwrap_or(Colour::black().into());
        let diffuse = parse_struct(yaml, "diffuse")?;
        let specular = parse_struct(yaml, "specular").unwrap_or(Colour::black().into());
        let alpha = parse_float(yaml, "alpha").unwrap_or(1.0);
        let ior = parse_float(yaml, "ior").unwrap_or(1.5);

        Ok(PhongMaterial::new(ambient, diffuse, specular, alpha, ior))
    }
}
