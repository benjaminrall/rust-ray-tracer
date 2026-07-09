use crate::cameras::CameraTrait;
use crate::core::{Ray, Scene};
use crate::drawing::FrameBuffer;
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{get_default_progress_bar, Vector, Vertex};
use rayon::prelude::*;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Full camera defined by a location, look-at and up vectors, and vertical field of view.
pub struct FullCamera {
    fov: f64, // Vertical field of view of the camera (in radians)

    u: Vector, // Horizontal view component unit
    v: Vector, // Vertical view component unit

    position: Vertex,      // Position of the camera
    to_view_plane: Vector, // Vector from the camera to the viewing plane
    focal_length: f64,     // Distance from the camera to the viewing plane
}

impl FullCamera {
    /// Creates a new `FullCamera` instance.
    ///
    /// # Arguments
    ///
    /// * `fov`: Vertical field of view of the camera (in degrees).
    /// * `position`: Position of the camera.
    /// * `look_at`: Position of the camera's target, at which the viewing plane will be placed.
    /// * `up`: Vector specifying the upwards direction.
    pub fn new(fov: f64, position: Vertex, look_at: Vertex, up: Vector) -> FullCamera {
        // Calculates the vector to the viewing plane and the focal length
        let to_view_plane = look_at - position;
        let focal_length = to_view_plane.length();

        // Creates orthonormal basis for the camera
        let w = -to_view_plane.unit();
        let u = Vector::cross(&up, &w).unit();
        let v = Vector::cross(&w, &u).unit();

        // Constructs camera struct
        FullCamera {
            fov: fov.to_radians(),
            u,
            v,
            position,
            to_view_plane,
            focal_length,
        }
    }

    /// Gets a ray from the camera to a specified pixel in an image.
    ///
    /// # Arguments
    ///
    /// * `px`: X position of the pixel.
    /// * `py`: Y position of the pixel.
    /// * `width`: Width of the image.
    /// * `height`: Height of the image.
    /// * `vw`: Width of the viewport.
    /// * `vh`: Height of the viewport.
    pub fn get_pixel_ray(
        &self,
        px: i32,
        py: i32,
        width: i32,
        height: i32,
        vw: f64,
        vh: f64,
    ) -> Ray {
        // Calculates position of the centre of the pixel on the image plane, as coordinates in the range [-1, 1]
        let fx = (vw * (px as f64 + 0.5) / width as f64) - (vw / 2.);
        let fy = (vh / 2.) - (vh * (py as f64 + 0.5) / height as f64);

        // Calculates direction of the outgoing ray
        let direction = fx * self.u + fy * self.v + self.to_view_plane;

        // Constructs the ray to be traced through the scene
        Ray::new(self.position, direction.unit())
    }
}

impl CameraTrait for FullCamera {
    fn render(self, scene: Scene, fb: &mut FrameBuffer) {
        // Gets the width and height of the FrameBuffer and viewport
        let (w, h) = (fb.get_width(), fb.get_height());
        let vh = 2.0 * self.focal_length * (0.5 * self.fov).tan();
        let vw = vh * (w / h) as f64;

        // Creates progress bar and vector of all output pixels
        let n_pixels = w * h;
        let progress_bar = get_default_progress_bar(n_pixels as u64);
        let mut pixels = vec![[0.0; 3]; n_pixels as usize];

        // Iterates through each pixel in parallel and records their colours
        pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            // Calculates position of the current pixel
            let y = i as i32 / w;
            let x = i as i32 % w;

            // Traces the pixel's ray through the scene and saves the recorded colour
            let ray = self.get_pixel_ray(x, y, w, h, vw, vh);
            let colour = scene.camera_trace(&ray);
            *pixel = [colour.r, colour.g, colour.b];

            // Increments progress bar
            progress_bar.inc(1);
        });

        // Writes recorded pixel information to the FrameBuffer
        for y in 0..h {
            for x in 0..w {
                let pixel = pixels[(y * w + x) as usize];
                fb.plot_pixel(x, y, pixel[0], pixel[1], pixel[2]);
            }
        }
    }
}

/// Implements loading a `FullCamera` from a YAML file.
impl FromYaml for FullCamera {
    fn from_yaml(yaml: &Yaml) -> Result<FullCamera, YamlPropertyError> {
        // Parses properties for the camera
        let fov = parse_float(yaml, "fov")?;
        let position = parse_struct(yaml, "position")?;
        let look_at = parse_struct(yaml, "look_at")?;
        let up = parse_struct(yaml, "up")?;

        // Returns the new camera instance
        Ok(FullCamera::new(fov, position, look_at, up))
    }
}
