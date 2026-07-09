use crate::cameras::{FullCamera, RealisticCamera, SimpleCamera};
use crate::core::Scene;
use crate::drawing::FrameBuffer;
use crate::utils::yaml::{parse_string, FromYaml, YamlPropertyError};
use enum_dispatch::enum_dispatch;
use yaml_rust::Yaml;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent all camera types.
pub enum Camera {
    SimpleCamera,
    FullCamera,
    RealisticCamera,
}

#[enum_dispatch(Camera)]
/// Trait which must be implemented by all camera models.
pub trait CameraTrait {
    /// Renders a scene to the given FrameBuffer.
    ///
    /// # Arguments
    ///
    /// * `scene`: Scene to be rendered by the camera.
    /// * `fb`: FrameBuffer to save results to.
    fn render(self, scene: Scene, fb: &mut FrameBuffer);
}

/// Implements loading `Camera` structs from a YAML file.
impl FromYaml for Camera {
    fn from_yaml(yaml: &Yaml) -> Result<Camera, YamlPropertyError> {
        // Parses camera type as a string
        let camera_type = parse_string(yaml, "type")?;

        // Matches the type to its respective object
        match camera_type.as_str() {
            "SimpleCamera" => Ok(SimpleCamera::from_yaml(yaml)?.into()),
            "FullCamera" => Ok(FullCamera::from_yaml(yaml)?.into()),
            "RealisticCamera" => Ok(RealisticCamera::from_yaml(yaml)?.into()),
            _ => Err(YamlPropertyError::invalid("type")),
        }
    }
}
