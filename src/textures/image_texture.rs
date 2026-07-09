use crate::core::Hit;
use crate::drawing::{srgb_to_linear, Colour, TexCoords};
use crate::textures::TextureTrait;
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use image::{DynamicImage, GenericImageView};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Texture that uses UV texture coordinates to apply image data to an object.
pub struct ImageTexture {
    data: DynamicImage,
}

impl ImageTexture {
    pub const COLOUR_SCALE_RECIP: f64 = 1.0 / 255.0;

    /// Creates a new image texture, loading the given file for the texture map.
    pub fn new(filename: &str) -> ImageTexture {
        ImageTexture {
            data: image::open(filename)
                .expect(format!("Unable to open image {}.", filename).as_str()),
        }
    }
}

impl TextureTrait for ImageTexture {
    fn get_colour_at(&self, hit: &Hit) -> Colour {
        // If the object has no texture coordinates, return black
        if hit.tex_coords.is_none() {
            return Colour::black();
        }

        // Otherwise, get a reference to the texture coordinates
        let tex_coords = hit.tex_coords.as_ref().unwrap();
        self.get_colour(tex_coords)
    }

    fn get_colour(&self, tex_coords: &TexCoords) -> Colour {
        // Converts texture coordinates into image coordinates
        let i = (tex_coords.u % 1.0 * self.data.width() as f64) as u32;
        let j = ((1.0 - (tex_coords.v % 1.0)) * self.data.height() as f64) as u32;

        // Gets the pixel corresponding to the given texture coordinates
        let pixel = self.data.get_pixel(i, j);

        // Returns the scaled pixel values as a colour
        let r = srgb_to_linear(pixel[0] as f64 * Self::COLOUR_SCALE_RECIP);
        let g = srgb_to_linear(pixel[1] as f64 * Self::COLOUR_SCALE_RECIP);
        let b = srgb_to_linear(pixel[2] as f64 * Self::COLOUR_SCALE_RECIP);

        Colour::new(r, g, b)
    }
}

/// Implements loading an `ImageTexture` from a YAML file.
impl FromYaml for ImageTexture {
    fn from_yaml(yaml: &Yaml) -> Result<ImageTexture, YamlPropertyError> {
        // Parses the filename for the texture
        let filename = parse_string(yaml, "filename")?;
        Ok(ImageTexture::new(&filename))
    }
}
