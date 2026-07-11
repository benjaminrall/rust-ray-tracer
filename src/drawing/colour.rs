use crate::textures::{ColourTexture, Texture};
use crate::utils::yaml::{parse_float, parse_float_array, parse_vec, FromYaml, YamlPropertyError};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};
use yaml_rust::Yaml;

/// Converts a linear colour value to sRGB space.
pub fn linear_to_srgb(l: f64) -> f64 {
    let l = l.clamp(0.0, 1.0);
    if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

/// Converts an sRGB colour value to linear space.
pub fn srgb_to_linear(s: f64) -> f64 {
    let s = s.clamp(0.0, 1.0);
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

#[derive(Debug, Clone, Copy)]
/// Struct to store and manipulate an RGB colour.
pub struct Colour {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Colour {
    /// Creates a new `Colour` instance with specified RGB values.
    pub fn new(r: f64, g: f64, b: f64) -> Colour {
        Colour { r, g, b }
    }

    /// Constructs a new `Colour` instance from a given list of RGB values.
    pub fn from_list(values: Vec<f64>) -> Colour {
        Colour {
            r: values[0],
            g: values[1],
            b: values[2],
        }
    }

    /// Creates a black `Colour` instance with values (0, 0, 0).
    pub fn black() -> Colour {
        Colour::new(0.0, 0.0, 0.0)
    }

    /// Creates a white `Colour` instance with values (1, 1, 1).
    pub fn white() -> Colour {
        Colour::new(1.0, 1.0, 1.0)
    }

    /// Returns the reciprocal of each of the colour's elements.
    pub fn reciprocal(&self) -> Colour {
        Colour::new(1.0 / self.r, 1.0 / self.g, 1.0 / self.b)
    }

    /// Scales the colour by another colour's values.
    pub fn scale_by_colour(&mut self, scaling: &Colour) {
        self.r *= scaling.r;
        self.g *= scaling.g;
        self.b *= scaling.b;
    }

    /// Scales the colour by a given float.
    pub fn scale_by_float(&mut self, scaling: f64) {
        self.r *= scaling;
        self.g *= scaling;
        self.b *= scaling;
    }

    /// Adds an adjustment to the colour.
    pub fn add(&mut self, adjust: &Colour) {
        self.r += adjust.r;
        self.g += adjust.g;
        self.b += adjust.b;
    }

    /// Returns the maximum value of the RGB components of the colour.
    pub fn max(&self) -> f64 {
        self.r.max(self.g.max(self.b))
    }

    /// Returns the L1 normalised colour.
    pub fn normalised(&self) -> Colour {
        let total = self.sum();
        if total == 0.0 {
            Colour::black()
        } else {
            Colour::new(self.r / total, self.g / total, self.b / total)
        }
    }

    /// Returns the sum of the colour's elements.
    pub fn sum(&self) -> f64 {
        self.r + self.g + self.b
    }

    /// Clamps the colour to a specified range.
    pub fn clamp(&self, min: f64, max: f64) -> Colour {
        Colour::new(
            self.r.clamp(min, max),
            self.g.clamp(min, max),
            self.b.clamp(min, max),
        )
    }
}

/// Implements addition of colours.
impl Add for Colour {
    type Output = Colour;

    fn add(self, other: Colour) -> Colour {
        Colour::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

/// Implements addition of colours with the '+=' operator.
impl AddAssign for Colour {
    fn add_assign(&mut self, other: Colour) {
        self.add(&other)
    }
}

/// Implements multiplication of colours.
impl Mul for Colour {
    type Output = Colour;

    fn mul(self, other: Colour) -> Colour {
        Colour::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}

/// Implements multiplication of colours with the '*=' operator.
impl MulAssign for Colour {
    fn mul_assign(&mut self, other: Colour) {
        self.scale_by_colour(&other)
    }
}

/// Implements multiplication of a colour by a scalar value.
impl Mul<f64> for Colour {
    type Output = Colour;

    fn mul(self, other: f64) -> Colour {
        Colour::new(self.r * other, self.g * other, self.b * other)
    }
}

/// Implements multiplication of a scalar value by a colour.
impl Mul<Colour> for f64 {
    type Output = Colour;

    fn mul(self, other: Colour) -> Colour {
        Colour::new(other.r * self, other.g * self, other.b * self)
    }
}

/// Implements multiplication of a colour by a scalar value with the '*=' operator.
impl MulAssign<f64> for Colour {
    fn mul_assign(&mut self, other: f64) {
        self.scale_by_float(other);
    }
}

/// Implements division of a colour by a scalar value.
impl Div<f64> for Colour {
    type Output = Colour;

    fn div(self, other: f64) -> Colour {
        Colour::new(self.r / other, self.g / other, self.b / other)
    }
}

/// Implements division of a colour by a scalar value with the '/=' operator.
impl DivAssign<f64> for Colour {
    fn div_assign(&mut self, other: f64) {
        self.r /= other;
        self.g /= other;
        self.b /= other;
    }
}

/// Implements implied conversion of colours to textures.
impl Into<Texture> for Colour {
    fn into(self) -> Texture {
        ColourTexture::new(self).into()
    }
}

/// Implements loading a `Colour` from a YAML file.
impl FromYaml for Colour {
    fn from_yaml(yaml: &Yaml) -> Result<Colour, YamlPropertyError> {
        // Attempts to construct the colour by treating the `Yaml` instance as either an array or hashmap
        if let Yaml::Array(array) = yaml {
            // Attempts to convert array into floats
            let props = parse_float_array(array)?;

            // Ensures the correct number of values were specified
            match props.len() {
                3 => Ok(Colour::from_list(props)),
                _ => Err(YamlPropertyError::invalid("array")),
            }
        } else if let Ok(srgb_array) = parse_vec(yaml, "srgb") {
            // Attempts to convert array into floats
            let props = parse_float_array(&srgb_array)?;

            // Ensures the correct number of values were specified
            match props.len() {
                3 => {
                    let r = srgb_to_linear(props[0]);
                    let g = srgb_to_linear(props[1]);
                    let b = srgb_to_linear(props[2]);
                    Ok(Colour::new(r, g, b))
                }
                _ => Err(YamlPropertyError::invalid("srgb")),
            }
        } else {
            // Parses named properties for the colour
            let r = parse_float(yaml, "r")?;
            let g = parse_float(yaml, "g")?;
            let b = parse_float(yaml, "b")?;

            // Returns the new colour instance
            Ok(Colour::new(r, g, b))
        }
    }
}
