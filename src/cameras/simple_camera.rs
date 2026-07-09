use crate::cameras::CameraTrait;
use crate::core::{Ray, Scene};
use crate::drawing::FrameBuffer;
use crate::utils::yaml::{parse_float, FromYaml, YamlPropertyError};
use crate::utils::{Vector, Vertex};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Simple camera that sits at the origin facing in the positive Z direction.
pub struct SimpleCamera {
    focal_length: f64, // Focal length of the camera
}

impl SimpleCamera {
    /// Creates a new `SimpleCamera` instance.
    pub fn new(focal_length: f64) -> SimpleCamera {
        SimpleCamera { focal_length }
    }

    /// Gets a ray from the camera to a specified pixel in an image.
    ///
    /// # Arguments
    ///
    /// * `px`: X position of the pixel.
    /// * `py`: Y position of the pixel.
    /// * `width`: Width of the image.
    /// * `height`: Height of the image.
    pub fn get_pixel_ray(&self, px: i32, py: i32, width: i32, height: i32) -> Ray {
        // Calculates position of the centre of the pixel relative to the camera's centre in the range [0, 1]
        let fx = (px as f64 + 0.5) / (width as f64);
        let fy = (py as f64 + 0.5) / (height as f64);

        // Calculates direction of the outgoing ray
        let direction = Vector::new(fx - 0.5, 0.5 - fy, self.focal_length);

        // Constructs the ray to be traced through the scene
        Ray::new(Vertex::zero(), direction.unit())
    }
}

impl CameraTrait for SimpleCamera {
    fn render(self, scene: Scene, fb: &mut FrameBuffer) {
        // Gets the width and height of the FrameBuffer
        let (w, h) = (fb.get_width(), fb.get_height());

        // Iterates through each pixel and writes their colours to the FrameBuffer
        for y in 0..h {
            for x in 0..w {
                let ray = self.get_pixel_ray(x, y, w, h);
                let colour = scene.camera_trace(&ray);
                fb.plot_pixel(x, y, colour.r, colour.g, colour.b);
            }
        }
    }
}

/// Implements loading a `SimpleCamera` from a YAML file.
impl FromYaml for SimpleCamera {
    fn from_yaml(yaml: &Yaml) -> Result<SimpleCamera, YamlPropertyError> {
        // Parses properties for the camera
        let focal_length = parse_float(yaml, "focal_length")?;

        // Returns the new camera instance
        Ok(SimpleCamera::new(focal_length))
    }
}
