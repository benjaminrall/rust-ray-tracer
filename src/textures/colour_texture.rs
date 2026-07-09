use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::TextureTrait;
use crate::utils::yaml::{parse_struct, FromYaml, YamlPropertyError};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Constant texture to represent a solid colour.
pub struct ColourTexture {
    colour: Colour,
}

impl ColourTexture {
    /// Creates a new texture from the given colour
    pub fn new(colour: Colour) -> ColourTexture {
        ColourTexture { colour }
    }
}

impl TextureTrait for ColourTexture {
    fn get_colour_at(&self, _hit: &Hit) -> Colour {
        self.colour
    }

    fn get_colour(&self, _tex_coords: &TexCoords) -> Colour {
        self.colour
    }
}

/// Implements loading a `ColourTexture` from a YAML file.
impl FromYaml for ColourTexture {
    fn from_yaml(yaml: &Yaml) -> Result<ColourTexture, YamlPropertyError> {
        // Parses the colour for the texture
        let colour = parse_struct(yaml, "colour")?;
        Ok(ColourTexture::new(colour))
    }
}
