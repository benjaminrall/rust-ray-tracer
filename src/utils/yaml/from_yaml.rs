use crate::utils::yaml::YamlPropertyError;
use yaml_rust::Yaml;

/// Trait for all objects that can be loaded from a YAML file.
pub trait FromYaml {
    /// Attempts to create an object from a loaded `Yaml` instance.
    fn from_yaml(yaml: &Yaml) -> Result<Self, YamlPropertyError>
    where
        Self: Sized;
}
