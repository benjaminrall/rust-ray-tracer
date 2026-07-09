use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::TextureTrait;
use crate::utils::yaml::{parse_float, FromYaml, YamlPropertyError};
use crate::utils::{layered_noise, noise, turbulence, Vertex};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Texture of a marble pattern generated using Perlin noise.
pub struct MarbleTexture {
    scale: f64,
}

impl MarbleTexture {
    /// Creates a new marble texture with a given scale.
    pub fn new(scale: f64) -> MarbleTexture {
        MarbleTexture { scale }
    }
}

impl TextureTrait for MarbleTexture {
    fn get_colour_at(&self, hit: &Hit) -> Colour {
        // Uses a sine function applied to turbulence to imitate a marble-like texture.
        let n = 0.5
            * (1.0
                + (self.scale * (hit.position.y + hit.position.x)
                    + 20. * turbulence(hit.position, 8, 0.5))
                .sin());

        // Colour bounds for the marble
        let k = 0.1;
        let w = 0.9;

        // Final scale for the marble
        let n = k + w * n;
        Colour::white() * n
    }

    fn get_colour(&self, _tex_coords: &TexCoords) -> Colour {
        Colour::black()
    }
}

/// Implements loading a `MarbleTexture` from a YAML file.
impl FromYaml for MarbleTexture {
    fn from_yaml(yaml: &Yaml) -> Result<MarbleTexture, YamlPropertyError> {
        let scale = parse_float(yaml, "scale")?;
        Ok(MarbleTexture { scale })
    }
}
