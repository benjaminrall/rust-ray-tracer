use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::colour_texture::ColourTexture;
use crate::textures::marble_texture::MarbleTexture;
use crate::textures::noise_texture::NoiseTexture;
use crate::textures::y_gradient_texture::YGradientTexture;
use crate::textures::{CheckerTexture, ImageTexture};
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use enum_dispatch::enum_dispatch;
use yaml_rust::Yaml;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent all textures.
pub enum Texture {
    ColourTexture,
    CheckerTexture,
    ImageTexture,
    NoiseTexture,
    MarbleTexture,
    YGradientTexture,
}

#[enum_dispatch(Texture)]
/// Trait which must be implemented by all `Texture` structs.
pub trait TextureTrait {
    /// Gets the colour of a texture for a given hit.
    fn get_colour_at(&self, hit: &Hit) -> Colour;

    /// Gets the colour of a texture for given texture coordinates.
    fn get_colour(&self, tex_coords: &TexCoords) -> Colour;
}

/// Implements loading `Texture` structs from a YAML file.
impl FromYaml for Texture {
    fn from_yaml(yaml: &Yaml) -> Result<Texture, YamlPropertyError> {
        // Checks if the texture has been represented directly as a colour
        if let Ok(colour) = Colour::from_yaml(yaml) {
            return Ok(ColourTexture::new(colour).into());
        }

        // Parses texture type as a String
        let texture_type = parse_string(yaml, "type")?;

        // Matches the type to its respective object
        match texture_type.as_str() {
            "ColourTexture" => Ok(ColourTexture::from_yaml(yaml)?.into()),
            "CheckerTexture" => Ok(CheckerTexture::from_yaml(yaml)?.into()),
            "ImageTexture" => Ok(ImageTexture::from_yaml(yaml)?.into()),
            "NoiseTexture" => Ok(NoiseTexture::from_yaml(yaml)?.into()),
            "MarbleTexture" => Ok(MarbleTexture::from_yaml(yaml)?.into()),
            "YGradientTexture" => Ok(YGradientTexture::from_yaml(yaml)?.into()),
            _ => Err(YamlPropertyError::invalid("type")),
        }
    }
}
