use crate::utils::yaml::{ExtendYamlResult, FromYaml, YamlPropertyError};
use crate::utils::Transform;
use std::ops::Index;
use yaml_rust::Yaml;

/// Attempts to parse a property from a given `Yaml` instance as an integer.
pub fn parse_int(yaml: &Yaml, property: &str) -> Result<i64, YamlPropertyError> {
    match yaml.index(property) {
        // Returns the value of the property if it's an integer
        Yaml::Integer(i) => Ok(*i),

        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Returns an error if any other invalid type is given
        _ => Err(YamlPropertyError::invalid(property)),
    }
}

/// Attempts to parse a property from a given `Yaml` instance as a float.
///
/// Implicitly converts integers into floats.
pub fn parse_float(yaml: &Yaml, property: &str) -> Result<f64, YamlPropertyError> {
    match yaml.index(property) {
        // Parses the property as a float
        Yaml::Real(r) => r.parse().map_err(|_| YamlPropertyError::invalid(property)),

        // Converts integer properties to floats implicitly
        Yaml::Integer(i) => Ok(*i as f64),

        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Returns an error if any other invalid type is given
        _ => Err(YamlPropertyError::invalid(property)),
    }
}

// Attempts to convert a list of `Yaml` instances to an array of floats.
///
/// Implicitly converts integers in the array into floats.
pub fn parse_float_array(array: &Vec<Yaml>) -> Result<Vec<f64>, YamlPropertyError> {
    array
        .iter()
        .map(|item| match item {
            // Parses float strings in the array
            Yaml::Real(rs) => rs.parse().map_err(|_| YamlPropertyError::invalid("array")),

            // Converts integers in the array into floats implicitly
            Yaml::Integer(i) => Ok(*i as f64),

            // Returns an error for any other invalid type within the array
            _ => Err(YamlPropertyError::invalid("array")),
        })
        .collect()
}

/// Attempts to parse a property from a given `Yaml` instance as a boolean.
pub fn parse_bool(yaml: &Yaml, property: &str) -> Result<bool, YamlPropertyError> {
    match yaml.index(property) {
        // Returns the value of the property if it's a boolean
        Yaml::Boolean(b) => Ok(*b),

        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Returns an error if any other invalid type is given
        _ => Err(YamlPropertyError::invalid(property)),
    }
}

/// Attempts to parse a property from a given `Yaml` instance as a string.
///
/// Implicitly converts integers, floats, and booleans into strings.
pub fn parse_string(yaml: &Yaml, property: &str) -> Result<String, YamlPropertyError> {
    match yaml.index(property) {
        // Returns the value of the property if it's a string
        Yaml::String(s) => Ok(s.clone()),

        // Converts ints, floats, and booleans to strings implicitly
        Yaml::Integer(i) => Ok(i.to_string()),
        Yaml::Real(r) => Ok(r.clone()),
        Yaml::Boolean(b) => Ok(b.to_string()),

        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Returns an error if any other invalid type is given
        _ => Err(YamlPropertyError::invalid(property)),
    }
}

/// Attempts to parse a property from a given `Yaml` instance as a vector of `Yaml`s.
pub fn parse_vec(yaml: &Yaml, property: &str) -> Result<Vec<Yaml>, YamlPropertyError> {
    match yaml.index(property) {
        // Returns the value of the property if it's an array
        Yaml::Array(v) => Ok(v.clone()),

        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Returns an error if any other invalid type is given
        _ => Err(YamlPropertyError::invalid(property)),
    }
}

/// Attempts to parse a property from a given `Yaml` instance as a generic struct.
pub fn parse_struct<T: FromYaml>(yaml: &Yaml, property: &str) -> Result<T, YamlPropertyError> {
    match yaml.index(property) {
        // Returns an error if the property is missing
        Yaml::BadValue => Err(YamlPropertyError::missing(property)),

        // Attempts to parse the property into the desired struct
        property_yaml => T::from_yaml(&property_yaml).extend_err(property),
    }
}

/// Attempts to parse a property from a given `Yaml` as an array of generic structs.
pub fn parse_struct_array<T: FromYaml>(
    yaml: &Yaml,
    property: &str,
) -> Result<Vec<T>, YamlPropertyError> {
    // Parses the property as an array of `Yaml`s
    let structs_yaml = parse_vec(yaml, property)?;

    // Loads each object from the array
    let mut structs = Vec::with_capacity(structs_yaml.len());
    for (i, struct_yaml) in structs_yaml.iter().enumerate() {
        let s = T::from_yaml(struct_yaml).extend_err(&format!("{}.{}", property, i))?;
        structs.push(s);
    }
    Ok(structs)
}

/// Attempts to parse the 'transforms' list property from a given `Yaml` instance.
///
/// Returns a single combined transform, created by applying each given transform in order.
pub fn parse_transforms(yaml: &Yaml) -> Result<Transform, YamlPropertyError> {
    let mut result = Transform::identity();

    // Attempts to get the list of transforms
    if let Ok(transforms) = parse_vec(yaml, "transforms") {
        // Iterates through the transforms and combines them
        for (i, t) in transforms.iter().enumerate() {
            let transform = Transform::from_yaml(t).extend_err(&format!("transforms.{}", i))?;
            result = transform * result;
        }
    }

    Ok(result)
}
