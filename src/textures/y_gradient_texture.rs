use crate::core::Hit;
use crate::drawing::{Colour, TexCoords};
use crate::textures::{CheckerTexture, TextureTrait};
use crate::utils::yaml::{
    parse_float, parse_float_array, parse_struct, parse_struct_array, parse_vec, FromYaml,
    YamlPropertyError,
};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Texture that generates a colour gradient based on a hit's `y` position.
pub struct YGradientTexture {
    scale: Colour,         // Scale to adjust the colour values by
    colours: Vec<Colour>,  // Colours making up the gradient
    percentages: Vec<f64>, // Percentage of the range taken up by each element

    min_range: f64,  // Minimum `y` value for the gradient range
    max_range: f64,  // Maximum 'y' value for the gradient range
    range_size: f64, // Size of the gradient range of `y` values
}

impl YGradientTexture {
    /// Creates a new Y gradient texture.
    pub fn new(
        scale: Colour,
        colours: Vec<Colour>,
        percentages: Vec<f64>,
        min_range: f64,
        max_range: f64,
    ) -> YGradientTexture {
        YGradientTexture {
            scale,
            colours,
            percentages,
            min_range,
            max_range,
            range_size: max_range - min_range,
        }
    }
}

impl TextureTrait for YGradientTexture {
    fn get_colour_at(&self, hit: &Hit) -> Colour {
        // Gets the y value from the hit
        let y = hit.position.y;

        // Checks that the y value lies within the texture's range
        if y < self.min_range || y > self.max_range {
            return Colour::black();
        }

        // Linearly interpolates based on the y value and colour proportions to return a soft gradient
        let mut y_proportion = (y - self.min_range) / self.range_size;
        let mut cumulative = 0.0;
        for i in 0..self.percentages.len() {
            cumulative += self.percentages[i];
            if y_proportion <= cumulative {
                if i == 0 {
                    return self.scale * self.colours[i];
                }
                let start = cumulative - self.percentages[i];
                let t = (y_proportion - start) / (cumulative - start);
                let c1 = self.colours[i - 1];
                let c2 = self.colours[i];
                let c3 = c1 * (1.0 - t) + c2 * t;
                return c3 * self.scale;
            }
        }
        self.scale * self.colours.last().unwrap().clone()
    }

    fn get_colour(&self, _tex_coords: &TexCoords) -> Colour {
        Colour::black()
    }
}

/// Implements loading a `YGradientTexture` from a YAML file.
impl FromYaml for YGradientTexture {
    fn from_yaml(yaml: &Yaml) -> Result<YGradientTexture, YamlPropertyError> {
        // Parses properties for the texture
        let scale = parse_struct(yaml, "scale")?;
        let colours = parse_struct_array(yaml, "colours")?;
        let percentages = parse_float_array(&parse_vec(yaml, "percentages")?)?;
        let min_range = parse_float(yaml, "min")?;
        let max_range = parse_float(yaml, "max")?;

        Ok(YGradientTexture::new(
            scale,
            colours,
            percentages,
            min_range,
            max_range,
        ))
    }
}
