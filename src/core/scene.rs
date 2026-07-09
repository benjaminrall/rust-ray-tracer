use crate::core::{Hit, Photon, Ray};
use crate::drawing::{Colour, TexCoords};
use crate::lights::{Light, LightTrait};
use crate::materials::{Material, MaterialTrait};
use crate::objects::{Object, ObjectTrait};
use crate::textures::{Texture, TextureTrait};
use crate::utils::yaml::{
    parse_int, parse_string, parse_struct, parse_struct_array, parse_vec, ExtendYamlResult,
    FromYaml, YamlPropertyError,
};
use crate::utils::{get_default_progress_bar, select_first, FilterType, PhotonMap, ScatterType};
use crate::{DIRECT_SAMPLES, MAX_INDIRECT_DEPTH, MAX_PHOTON_TRACE_DEPTH, MAX_RECURSE};
use rayon::prelude::*;
use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Struct to represent a scene, which is a database of objects and lights
/// which allow rays to be traced through it.
pub struct Scene {
    // Materials HashMap for all materials used by objects in the scene
    materials: HashMap<String, Material>,

    objects: Vec<Object>, // List of objects contained within the scene
    lights: Vec<Light>,   // List of lights in the scene

    background: Colour, // Background colour for the scene

    // Photon maps for the scene
    global_map: Option<PhotonMap>,
    caustic_map: Option<PhotonMap>,
    volume_map: Option<PhotonMap>,
}

impl Scene {
    /// Creates a new `Scene` instance from lists of existing objects and lights, and an existing materials hash map.
    pub fn new(
        materials: HashMap<String, Material>,
        objects: Vec<Object>,
        lights: Vec<Light>,
        background: Colour,
    ) -> Scene {
        Scene {
            materials,
            objects,
            lights,
            background,
            global_map: None,
            caustic_map: None,
            volume_map: None,
        }
    }

    /// Creates a new empty `Scene` instance with a black background.
    pub fn empty() -> Scene {
        Scene {
            materials: HashMap::new(),
            objects: vec![],
            lights: vec![],
            background: Colour::black(),
            global_map: None,
            caustic_map: None,
            volume_map: None,
        }
    }

    /// Creates a new empty `Scene` instance with a specified background colour.
    pub fn with_background(background: Colour) -> Scene {
        Scene {
            materials: HashMap::new(),
            objects: vec![],
            lights: vec![],
            background,
            global_map: None,
            caustic_map: None,
            volume_map: None,
        }
    }

    /// Adds a material to the scene.
    pub fn add_material(&mut self, name: &str, material: Material) {
        self.materials.insert(String::from(name), material);
    }

    /// Adds an object to the scene.
    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Adds a light to the scene.
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Returns the powers of each light source in the scene, along with their total combined power.
    fn get_light_powers(&self) -> (Vec<f64>, f64) {
        // Variables to store the total power and light powers
        let mut total_power = 0.0;
        let mut powers = Vec::with_capacity(self.lights.len());

        // Computes powers of each light
        for light in self.lights.iter() {
            // The power of a light is its sum (to reverse L1 normalisation) multiplied by its area
            let power = light.get_intensity().sum() * light.get_area();
            total_power += power;
            powers.push(power);
        }

        (powers, total_power)
    }

    /// Distributes `n` photons among the light sources in the scene, based on their powers.
    ///
    /// Returns a vector containing the photon counts for each light.
    fn distribute_photons(&self, n: usize, powers: &Vec<f64>, total_power: f64) -> Vec<usize> {
        // Computes number of photons per light
        let mut photon_counts = Vec::with_capacity(self.lights.len());
        for power in powers.iter() {
            let photons = (power / total_power) * n as f64;
            photon_counts.push(photons as usize);
        }
        photon_counts
    }

    /// Builds a photon map with at least `n` photons, using a given photon tracing function.
    pub fn build_photon_map<F>(&self, n: usize, trace_fn: F) -> PhotonMap
    where
        F: Fn(&Light) -> Vec<Photon> + Send + Sync,
    {
        // List to store all generated photons
        let mut photons = Vec::new();

        // Creates progress bar for photon tracing
        let progress_bar = get_default_progress_bar(n as u64);

        // Gets the powers of each light and the total power in the scene
        let (powers, total_power) = self.get_light_powers();

        // Calculates batch size of photons to be emitted each iteration, set to an underestimate of a half for simplicity.
        let batch_size = n / 2;

        // Emits photons from every light, proportional to their power, until the target photon number is reached
        let mut emitted = 0;
        while photons.len() < n {
            // Gets the number of photons to be emitted by each light
            let photon_counts = self.distribute_photons(batch_size, &powers, total_power);

            // Emits and stores photons
            for (light, count) in self.lights.iter().zip(photon_counts) {
                photons.extend(
                    (0..count)
                        .into_par_iter()
                        .flat_map(|_| {
                            // Traces a singular photon from the light source
                            let photons = trace_fn(light);

                            // Increments progress
                            progress_bar.inc(photons.len() as u64);

                            // Returns all photon records created by the photon
                            photons
                        })
                        .collect::<Vec<Photon>>(),
                );

                // Keeps track of the number of emitted photons to scale photon power
                emitted += count;
            }
        }

        // Scales the photons to account for the total scene power and the total number of emitted photons
        let progress_bar = get_default_progress_bar(photons.len() as u64);
        let photon_multiplier = total_power / emitted as f64;
        photons.par_iter_mut().for_each(|photon| {
            photon.scale_power(photon_multiplier);
            progress_bar.inc(1);
        });

        // Constructs and returns the photon map
        PhotonMap::new(photons)
    }

    /// Builds the global photon map for the scene.
    ///
    /// # Arguments
    ///
    /// * `n`: The target number of photons to store.
    pub fn build_global_map(&mut self, n: usize) {
        self.global_map = Some(self.build_photon_map(n, |light| self.trace_global_photon(light)))
    }

    /// Builds the caustic photon map for the scene.
    ///
    /// # Arguments
    ///
    /// * `n`: The target number of photons to store.
    pub fn build_caustic_map(&mut self, n: usize) {
        self.caustic_map = Some(self.build_photon_map(n, |light| self.trace_caustic_photon(light)))
    }

    /// Builds the volume photon map for the scene.
    ///
    /// # Arguments
    ///
    /// * `n`: The target number of photons to store.
    pub fn build_volume_map(&mut self, n: usize) {
        self.volume_map = Some(self.build_photon_map(n, |light| self.trace_volume_photon(light)))
    }

    /// Traces a photon from the given light source through the scene, to be stored in the global
    /// photon map.
    ///
    /// Stores all photon paths of the form L{S|D|V}*D (in Heckbert's notation), which represents
    /// the global illumination for all diffuse surfaces in the scene.
    pub fn trace_global_photon(&self, light: &Light) -> Vec<Photon> {
        // Initialises photon list and the sampled photon's information
        let mut photons = Vec::new();
        let mut photon_ray = light.sample_photon_ray();

        // Power is normalised to be later scaled by the scene's total power
        let mut power = light.get_intensity().normalised();

        // Recursively traces the photon until it doesn't hit anything, is absorbed, or reaches its maximum depth
        for _ in 0..MAX_PHOTON_TRACE_DEPTH {
            // Gets the first object hit by the photon
            if let Some(hit) = self.trace(&photon_ray) {
                // Gets the material of the hit object
                let material = self.materials.get(hit.material).expect(&format!(
                    "Material '{}' has not been added to the scene.",
                    hit.material
                ));

                // Scatters the photon based on the material's properties
                if let Some((new_ray, scatter_type)) =
                    material.scatter_photon(&photon_ray, &hit, &mut power)
                {
                    // Stores the photon if there was a diffuse interaction
                    if scatter_type == ScatterType::Diffuse {
                        photons.push(Photon::new(hit.position, -photon_ray.direction, power))
                    }

                    // Updates the photon ray to its scattered direction
                    photon_ray = new_ray
                } else {
                    // Breaks if the photon was absorbed
                    break;
                }
            } else {
                // Breaks if nothing was hit by the photon
                break;
            }
        }

        // Returns the list of all stored photons from the trace
        photons
    }

    /// Traces a photon from the given light source through the scene, to be stored in the caustic
    /// photon map.
    ///
    /// Stores all photon paths of the form LS+D (in Heckbert's notation), which represents photons
    /// that have been through one or more specular reflections before hitting a diffuse surface.
    pub fn trace_caustic_photon(&self, light: &Light) -> Vec<Photon> {
        // Initialises photon list and the sampled photon's information
        let mut photons = Vec::new();
        let mut photon_ray = light.sample_photon_ray();

        // Power is normalised to be later scaled by the scene's total power
        let mut power = light.get_intensity().normalised();

        // Flag to ensure photon hits aren't stored until a specular surface is hit
        let mut store = false;

        // Recursively traces the photon until it doesn't hit anything, is absorbed, or reaches its maximum depth
        for _ in 0..MAX_PHOTON_TRACE_DEPTH {
            // Gets the first object hit by the photon
            if let Some(hit) = self.trace(&photon_ray) {
                // Gets the material of the hit object
                let material = self.materials.get(hit.material).expect(&format!(
                    "Material '{}' has not been added to the scene.",
                    hit.material
                ));

                // Scatters the photon based on the material's properties
                if let Some((new_ray, scatter_type)) =
                    material.scatter_photon(&photon_ray, &hit, &mut power)
                {
                    if scatter_type == ScatterType::Diffuse && store {
                        // Stores the photon if it hit a diffuse surface after hitting a specular surface
                        photons.push(Photon::new(hit.position, -photon_ray.direction, power));
                        break;
                    } else if scatter_type == ScatterType::Specular {
                        // Enables storing of photons if a specular surface was hit
                        store = true;
                    } else {
                        // If any other interaction took place, the photon is no longer relevant
                        break;
                    }

                    // Updates the photon ray to its scattered direction
                    photon_ray = new_ray;
                } else {
                    // Breaks if the photon was absorbed
                    break;
                }
            } else {
                // Breaks if nothing was hit by the photon
                break;
            }
        }

        // Returns the list of all stored photons from the trace
        photons
    }

    /// Traces a photon from the given light source through the scene, to be stored in the volume
    /// photon map.
    ///
    /// Stores all photon paths of the form L{S|D|V}+V (in Heckbert's notation), which represents
    /// photons that have hit a scattering volume indirectly (after scattering from any other surface).
    pub fn trace_volume_photon(&self, light: &Light) -> Vec<Photon> {
        // Initialises photon list and the sampled photon's information
        let mut photons = Vec::new();
        let mut photon_ray = light.sample_photon_ray();

        // Power is normalised to be later scaled by the scene's total power
        let mut power = light.get_intensity().normalised();

        // Flag to ensure photon hits aren't stored until any surface is hit directly
        let mut store = false;

        // Recursively traces the photon until it doesn't hit anything, is absorbed, or reaches its maximum depth
        for _ in 0..MAX_PHOTON_TRACE_DEPTH {
            // Gets the first object hit by the photon
            if let Some(hit) = self.trace(&photon_ray) {
                // Gets the material of the hit object
                let material = self.materials.get(hit.material).expect(&format!(
                    "Material '{}' has not been added to the scene.",
                    hit.material
                ));

                // Scatters the photon based on the material's properties
                if let Some((new_ray, scatter_type)) =
                    material.scatter_photon(&photon_ray, &hit, &mut power)
                {
                    // Stores the photon if it hit a volume indirectly
                    if scatter_type == ScatterType::Volume && store {
                        photons.push(Photon::new(hit.position, -photon_ray.direction, power))
                    }

                    // Enables storing of photons after any scattering event
                    store = true;

                    // Updates the photon ray to its scattered direction
                    photon_ray = new_ray;
                } else {
                    // Breaks if the photon was absorbed
                    break;
                }
            } else {
                // Breaks if nothing was hit by the photon
                break;
            }
        }

        // Returns the list of all stored photons from the trace
        photons
    }

    /// Computes a radiance estimate from the global map, based on a given hit and function,
    /// defined by a material, that maps photons to colours.
    pub fn global_radiance_estimate<F>(&self, hit: &Hit, photon_colour_fn: F) -> Colour
    where
        F: Fn(&Photon) -> Colour,
    {
        // Checks that the global map exists
        if self.global_map.is_none() {
            return Colour::black();
        }

        // Calculates the global map's radiance estimate
        self.global_map.as_ref().unwrap().estimate_radiance(
            hit,
            photon_colour_fn,
            50,
            2.5,
            FilterType::Disk,
        )
    }

    /// Computes a radiance estimate from the caustic map, based on a given hit and function,
    /// defined by a material, that maps photons to colours.
    pub fn caustic_radiance_estimate<F>(&self, hit: &Hit, photon_colour_fn: F) -> Colour
    where
        F: Fn(&Photon) -> Colour,
    {
        // Checks that the caustic map exists
        if self.caustic_map.is_none() {
            return Colour::black();
        }

        // Calculates the caustic map's radiance estimate
        self.caustic_map.as_ref().unwrap().estimate_radiance(
            hit,
            photon_colour_fn,
            225,
            0.225,
            FilterType::Cone,
        )
    }

    /// Computes a radiance estimate from the volume map, based on a given hit and function,
    /// defined by a material, that maps photons to colours.
    pub fn volume_radiance_estimate<F>(&self, hit: &Hit, photon_colour_fn: F) -> Colour
    where
        F: Fn(&Photon) -> Colour,
    {
        // Checks that the volume map exists
        if self.volume_map.is_none() {
            return Colour::black();
        }

        // Calculates the volume map's radiance estimate
        self.volume_map.as_ref().unwrap().estimate_radiance(
            hit,
            photon_colour_fn,
            500,
            1.0,
            FilterType::Sphere,
        )
    }

    /// Traces a ray through the scene and returns the first hit, if one exists.
    pub fn trace(&self, ray: &Ray) -> Option<Hit> {
        self.objects
            .iter()
            .filter_map(|object| select_first(object.intersection(ray)))
            .min_by(|hit1, hit2| hit1.t.partial_cmp(&hit2.t).unwrap_or(Equal))
    }

    /// Traces a Monte Carlo ray through the scene until it hits a diffuse surface
    /// for accurate indirect lighting computation.
    pub fn indirect_raytrace(&self, mut ray: Ray) -> Colour {
        // Initialises the weight of the colour, used in place of a photon's power, to model scattering behaviour
        let mut weight = Colour::white();

        // Recursively traces the ray until it doesn't hit anything, is absorbed, or reaches its maximum depth
        for _ in 0..MAX_INDIRECT_DEPTH {
            // Gets the first object hit by the ray
            if let Some(hit) = self.trace(&ray) {
                // Gets the material of the hit object
                let material = self.materials.get(hit.material).expect(&format!(
                    "Material '{}' has not been added to the scene.",
                    hit.material
                ));

                // Scatters the ray as if it were a photon, based on the material's properties
                if let Some((new_ray, scatter_type)) =
                    material.scatter_photon(&ray, &hit, &mut weight)
                {
                    if scatter_type == ScatterType::Diffuse {
                        // Computes and returns the weighted global radiance estimate for the hit
                        return self.global_radiance_estimate(&hit, |photon| {
                            material.compute_per_photon(&ray, &hit, photon)
                        }) * weight;
                    }

                    // Updates the ray to its scattered direction
                    ray = new_ray;
                } else {
                    // Breaks if the ray was absorbed
                    break;
                }
            } else {
                // Breaks if nothing was hit by the ray
                break;
            }
        }

        // Returns black if no diffuse surface was hit
        Colour::black()
    }

    /// Traces a ray through the scene and returns its colour.
    pub fn raytrace(&self, ray: &Ray, recurse: i32, direct: bool) -> Colour {
        // Traces the ray through the scene and gets the first hit, if one exists
        if let Some(hit) = self.trace(&ray) {
            // Initialises colour accumulator for the surface's lighting
            let mut colour = Colour::black();

            // Gets the material of the hit object
            let material = self.materials.get(hit.material).expect(&format!(
                "Material '{}' has not been added to the scene.",
                hit.material
            ));

            // Adds the per intersection contribution to the colour
            colour += material.compute_once(&ray, &hit, recurse, self, direct);

            // Adds the contribution of each light to the colour
            for light in self.lights.iter() {
                // Adds the direct lighting component of the material for the light source
                colour += material.compute_per_light(ray, &hit, light, self, direct);
            }

            // Returns the sum of all colour contributions
            colour
        } else {
            // If no surface was hit, returns the background colour
            self.background
        }
    }

    /// Starts tracing a camera ray through the scene, returning its colour.
    ///
    /// Acts as a wrapper for the `raytrace` function, using the default max recurse value and
    /// with `direct` set to true.
    pub fn camera_trace(&self, ray: &Ray) -> Colour {
        self.raytrace(ray, MAX_RECURSE, true)
    }

    /// Traces a shadow ray through the scene and returns whether there was a hit.
    pub fn shadow_trace(&self, ray: &Ray, limit: f64) -> bool {
        // Loops through each object in the scene
        self.objects.iter().any(|object| {
            // If an object was hit within the shadow ray's limit's, returns true
            if let Some(hit) = select_first(object.intersection(&ray)) {
                hit.t >= 0. && hit.t < limit
            } else {
                false
            }
        })
    }
}

/// Implements loading a `Scene` from a YAML file.
impl FromYaml for Scene {
    fn from_yaml(yaml: &Yaml) -> Result<Scene, YamlPropertyError> {
        // Loads the hash map of materials
        let materials_yaml = parse_vec(yaml, "materials")?;
        let mut materials = HashMap::with_capacity(materials_yaml.len());
        for (i, material_yaml) in materials_yaml.iter().enumerate() {
            let name = parse_string(material_yaml, "name")
                .extend_err(format!("materials.{}", i).as_str())?;
            let material = Material::from_yaml(material_yaml)
                .extend_err(format!("materials.{}", i).as_str())?;

            materials.insert(name, material);
        }

        // Loads the vectors of objects and lights
        let objects = parse_struct_array(yaml, "objects")?;
        let lights = parse_struct_array(yaml, "lights")?;

        // Gets the optional background colour (defaults to black)
        let background = parse_struct(yaml, "background").unwrap_or(Colour::black());

        // Creates the scene instance
        let mut scene = Scene::new(materials, objects, lights, background);

        // Gets target number of global photons to be stored and creates global photon map
        if let Ok(global_photons) = parse_int(yaml, "global_photons") {
            if global_photons <= 0 {
                Err(YamlPropertyError::invalid("global_photons"))?
            }
            scene.build_global_map(global_photons as usize);
        }

        // Gets target number of caustic photons to be stored and creates caustic photon map
        if let Ok(caustic_photons) = parse_int(yaml, "caustic_photons") {
            if caustic_photons <= 0 {
                Err(YamlPropertyError::invalid("caustic_photons"))?
            }
            scene.build_caustic_map(caustic_photons as usize);
        }

        // Gets target number of volume photons to be stored and creates volume photon map
        if let Ok(volume_photons) = parse_int(yaml, "volume_photons") {
            if volume_photons <= 0 {
                Err(YamlPropertyError::invalid("volume_photons"))?
            }
            scene.build_volume_map(volume_photons as usize);
        }

        // Returns the constructed scene
        Ok(scene)
    }
}
