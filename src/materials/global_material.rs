use crate::core::{Hit, Photon, Ray, Scene};
use crate::drawing::Colour;
use crate::lights::Light;
use crate::materials::MaterialTrait;
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{normalise_colour_coefficients, random_float, ScatterType, Vector};
use crate::{AIR_IOR, SPECULAR_SAMPLES};
use yaml_rust::Yaml;
use crate::textures::{TextureTrait, Texture};

#[derive(Debug)]
/// Material that supports both reflection and refraction.
pub struct GlobalMaterial {
    reflect_weight: Texture, // Weighting of reflection
    refract_weight: Texture, // Weighting of refraction

    alpha: f64, // Phong shininess coefficient
    ior: f64,   // Index of refraction
}

impl GlobalMaterial {
    /// Creates a new `GlobalMaterial` instance.
    ///
    /// # Arguments
    ///
    /// * `reflect_weight`: Weighting of the material's reflection.
    /// * `refract_weight`: Weighting of the material's refraction.
    /// * `alpha`: Phong shininess coefficient for the material.
    /// * `ior`: Index of refraction of the material.
    pub fn new(
        reflect_weight: Texture,
        refract_weight: Texture,
        alpha: f64,
        ior: f64,
    ) -> GlobalMaterial {
        GlobalMaterial {
            reflect_weight,
            refract_weight,
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

    /// Uses Monte Carlo sampling to compute the colour of glossy reflection/refraction around an
    /// ideal reflected direction.
    ///
    /// # Arguments
    ///
    /// * `hit`: Point of intersection to emit reflected rays from.
    /// * `ideal_dir`: Ideal direction of reflection.
    /// * `cos_i`: Cosine of the angle of incidence.
    /// * `recurse`: Current recurse value to be reduced for recursive raytrace calls.
    /// * `scene`: Reference to the scene in order to access the raytrace method.
    /// * `direct`: Whether the hit was directly from the camera or through another Monte Carlo process.
    /// * `inside`: Whether the outgoing rays will be inside an object.
    fn compute_glossy_colour(
        &self,
        hit: &Hit,
        ideal_dir: &Vector,
        cos_i: f64,
        recurse: i32,
        scene: &Scene,
        direct: bool,
        inside: bool,
    ) -> Colour {
        // Calculates the number of Monte Carlo rays to be used for sampling the glossy colour
        let monte_carlo_rays = if direct { SPECULAR_SAMPLES } else { 1 };

        let mut colour = Colour::black();
        for _ in 0..monte_carlo_rays {
            let sample_dir = Vector::sample_specular_vector(ideal_dir, cos_i, self.alpha);

            // Traces the Monte Carlo ray to get its colour contribution
            let ray = Ray::offset_inside(hit.position, sample_dir, inside);
            colour += scene.raytrace(&ray, recurse - 1, false);
        }
        colour / monte_carlo_rays as f64
    }
}

impl MaterialTrait for GlobalMaterial {
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

        // Gets refractive indices for the hit
        let (n1, n2, nr) = self.calculate_refractive_indices(hit);

        // Calculates angle values for the hit from Snell's law
        let cos_i = -Vector::dot(&hit.normal, &incident.direction);
        let sin_t_sqr = nr * nr * (1.0 - cos_i * cos_i);
        let cos_t = (1. - sin_t_sqr).sqrt();

        // Computes Fresnel reflectance using Schlick's approximation
        let reflectance = Self::schlick(n1, n2, cos_i, sin_t_sqr, cos_t);

        // Shoots reflected rays into the scene and calculates their colour contribution
        let reflect_dir = incident.direction + 2.0 * cos_i * hit.normal;
        let reflection =
            self.compute_glossy_colour(hit, &reflect_dir, cos_i, recurse, scene, direct, false);

        // Shoots refracted rays into the scene, if they exist, and calculates their colour contribution
        let refraction = if sin_t_sqr > 1.0 {
            Colour::black()
        } else {
            let refract_dir = nr * incident.direction + (nr * cos_i - cos_t) * hit.normal;
            self.compute_glossy_colour(
                hit,
                &refract_dir,
                cos_i,
                recurse,
                scene,
                direct,
                hit.entering,
            )
        };

        // Normalises the reflection and refraction coefficients to make them conserve energy
        let mut coeffs = vec![
            self.reflect_weight.get_colour_at(hit),
            self.refract_weight.get_colour_at(hit)
        ];
        normalise_colour_coefficients(&mut coeffs);
        let reflect_weight = coeffs[0];
        let refract_weight = coeffs[1];

        // Computes colour based on the reflect and refract weights combined with the Fresnel reflectance term
        (reflect_weight + reflectance * refract_weight) * reflection
            + (1.0 - reflectance) * refract_weight * refraction
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

        // Normalises the reflection and refraction coefficients to make them conserve energy
        let mut coeffs = vec![
            self.reflect_weight.get_colour_at(hit),
            self.refract_weight.get_colour_at(hit)
        ];
        normalise_colour_coefficients(&mut coeffs);
        let reflect_weight = coeffs[0];
        let refract_weight = coeffs[1];

        // Computes probabilities of reflection and refraction based on their weights and Fresnel reflectance
        let refract_max = refract_weight.max();
        let reflect_prob = reflect_weight.max() + reflectance * refract_max;
        let refract_prob = (1.0 - reflectance) * refract_max;

        // Selects whether to reflect or refract the photon using Russian roulette based on these probabilities
        let eta = random_float();
        if eta < reflect_prob {
            // Calculates the reflected direction of the photon
            let ideal_dir = photon_ray.direction + 2.0 * cos_i * hit.normal;
            let reflect_dir = Vector::sample_specular_vector(&ideal_dir, cos_i, self.alpha);

            // Scales the photon's power by its probability of reflection and reflected colour
            *power *= (reflect_weight + reflectance * refract_weight) / reflect_prob;

            Some((
                Ray::offset(hit.position, reflect_dir),
                ScatterType::Specular,
            ))
        } else if eta < reflect_prob + refract_prob {
            // Calculates the refracted direction of the photon
            let ideal_dir = nr * photon_ray.direction + (nr * cos_i - cos_t) * hit.normal;
            let refract_dir = Vector::sample_specular_vector(&ideal_dir, cos_i, self.alpha);

            // Scales the photon's power by its probability of refraction and refracted colour
            *power *= refract_weight * ((1.0 - reflectance) / refract_prob);

            Some((
                Ray::offset_inside(hit.position, refract_dir, hit.entering),
                ScatterType::Specular,
            ))
        } else {
            // Returns `None` if the photon is absorbed
            None
        }
    }
}

/// Implements loading a `GlobalMaterial` from a YAML file.
impl FromYaml for GlobalMaterial {
    fn from_yaml(yaml: &Yaml) -> Result<GlobalMaterial, YamlPropertyError> {
        // Parses properties for the material
        let reflect_weight = parse_struct(yaml, "reflect_weight").unwrap_or(Colour::black().into());
        let refract_weight = parse_struct(yaml, "refract_weight").unwrap_or(Colour::black().into());
        let alpha = parse_float(yaml, "alpha").unwrap_or(10000.0);
        let ior = parse_float(yaml, "ior").unwrap_or(1.0);

        // Returns the new material instance
        Ok(GlobalMaterial::new(
            reflect_weight,
            refract_weight,
            alpha,
            ior,
        ))
    }
}
