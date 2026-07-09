use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
/// Struct for representing errors when parsing properties in YAML files.
pub struct YamlPropertyError {
    message: String,
    property_chain: String,
}

impl YamlPropertyError {
    /// Creates a new error with a given message and property.
    pub fn new(message: &str, property: &str) -> YamlPropertyError {
        YamlPropertyError {
            message: String::from(message),
            property_chain: String::from(property),
        }
    }

    /// Creates a new error for a missing property.
    pub fn missing(property: &str) -> YamlPropertyError {
        YamlPropertyError {
            message: format!("Missing property '{}' in YAML file.", property),
            property_chain: String::from(property),
        }
    }

    /// Creates a new error for a property with an incorrect type.
    pub fn invalid(property: &str) -> YamlPropertyError {
        YamlPropertyError {
            message: format!("Invalid type for property '{}' in YAML file.", property),
            property_chain: String::from(property),
        }
    }

    /// Adds a parent property to the chain.
    pub fn add(mut self, property: &str) -> YamlPropertyError {
        self.property_chain = format!("{}.{}", property, self.property_chain);
        self
    }
}

/// Implements displaying a `YamlPropertyError` for easy debugging.
impl Display for YamlPropertyError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}\nSource: {}", self.message, self.property_chain)
    }
}

/// Allows a `YamlPropertyError` to be treated as an error.
impl Error for YamlPropertyError {}

/// Trait to allow extending the `Result` type for easier property chaining.
pub trait ExtendYamlResult<T> {
    /// Extends a contained `YamlPropertyError`'s property chain, leaving an `Ok` value untouched.
    fn extend_err(self, property: &str) -> Result<T, YamlPropertyError>;
}

/// Allows properties to be added to the property chain directly through a `Result`.
impl<T> ExtendYamlResult<T> for Result<T, YamlPropertyError> {
    fn extend_err(self, property: &str) -> Result<T, YamlPropertyError> {
        self.map_err(|e| e.add(property))
    }
}
