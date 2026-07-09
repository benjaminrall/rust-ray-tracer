use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::TextureTrait;
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Texture that gives a spatial 3-dimensional checker pattern.
pub struct CheckerTexture {
    scale_recip: f64, // Reciprocal of the checker's scale

    even: Colour, // Colour to show in even positions
    odd: Colour,  // Colour to show in odd positions
}

impl CheckerTexture {
    /// Creates a new checker texture with the given scale and colours.
    pub fn new(scale: f64, even: Colour, odd: Colour) -> CheckerTexture {
        CheckerTexture {
            scale_recip: 1.0 / scale,
            even,
            odd,
        }
    }
}

impl TextureTrait for CheckerTexture {
    fn get_colour_at(&self, hit: &Hit) -> Colour {
        // Gets the floor of the scaled x, y, and z positions
        let x = (self.scale_recip * hit.position.x).floor() as i64;
        let y = (self.scale_recip * hit.position.y).floor() as i64;
        let z = (self.scale_recip * hit.position.z).floor() as i64;

        // Determines whether to return the odd or even colour
        if (x + y + z) % 2 == 0 {
            self.even
        } else {
            self.odd
        }
    }

    fn get_colour(&self, _tex_coords: &TexCoords) -> Colour {
        Colour::black()
    }
}

/// Implements loading a `CheckerTexture` from a YAML file.
impl FromYaml for CheckerTexture {
    fn from_yaml(yaml: &Yaml) -> Result<CheckerTexture, YamlPropertyError> {
        // Parses properties for the texture
        let scale = parse_float(yaml, "scale")?;
        let even = parse_struct(yaml, "even")?;
        let odd = parse_struct(yaml, "odd")?;

        Ok(CheckerTexture::new(scale, even, odd))
    }
}
