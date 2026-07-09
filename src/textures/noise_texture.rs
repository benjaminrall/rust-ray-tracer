use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::TextureTrait;
use crate::utils::layered_noise;
use crate::utils::yaml::{parse_float, FromYaml, YamlPropertyError};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Texture of a random pattern generated using Perlin noise.
pub struct NoiseTexture {
    scale: f64,
}

impl NoiseTexture {
    /// Creates a new noise texture with a given scale.
    pub fn new(scale: f64) -> NoiseTexture {
        NoiseTexture { scale }
    }
}

impl TextureTrait for NoiseTexture {
    fn get_colour_at(&self, hit: &Hit) -> Colour {
        // Uses simple layered perlin noise for a texture
        Colour::white() * 0.5 * (1.0 + layered_noise(self.scale * hit.position, 6, 0.5))
    }

    fn get_colour(&self, _tex_coords: &TexCoords) -> Colour {
        Colour::black()
    }
}

/// Implements loading a `NoiseTexture` from a YAML file.
impl FromYaml for NoiseTexture {
    fn from_yaml(yaml: &Yaml) -> Result<NoiseTexture, YamlPropertyError> {
        let scale = parse_float(yaml, "scale")?;
        Ok(NoiseTexture { scale })
    }
}
