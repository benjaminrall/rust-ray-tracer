use crate::cameras::CameraTrait;
use crate::core::{Ray, Scene};
use crate::drawing::{linear_to_srgb, FrameBuffer};
use crate::utils::gui::RayTracerGUI;
use crate::utils::yaml::{parse_float, parse_struct, FromYaml, YamlPropertyError};
use crate::utils::{random_float, random_in_unit_disk, Vector, Vertex};
use rayon::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Realistic camera using the thin lens approximation for depth of field.
pub struct RealisticCamera {
    fov: f64, // Vertical field of view of the camera (in radians)

    u: Vector, // Horizontal view component unit
    v: Vector, // Vertical view component unit

    position: Vertex,      // Position of the camera
    to_view_plane: Vector, // Vector from the camera to the viewing plane
    focal_length: f64,     // Distance from the camera to the viewing plane
    lens_radius: f64,      // Radius of the camera's lens

    pixels: Option<Vec<[f64; 3]>>, // Pixel information for continuing rendering from a PNG file
    samples: usize,                // Starting sample count for continuing rendering from a PNG file
}

impl RealisticCamera {
    /// Creates a new `RealisticCamera` instance.
    ///
    /// # Arguments
    ///
    /// * `fov`: Vertical field of view of the camera (in degrees).
    /// * `position`: Position of the camera.
    /// * `look_at`: Position of the camera's target.
    /// * `up`: Vector specifying the upwards direction.
    /// * `focal_length`: Distance from the camera to the viewing plane.
    /// * `aperture`: Size of the camera's aperture (diameter of the lens).
    pub fn new(
        fov: f64,
        position: Vertex,
        look_at: Vertex,
        up: Vector,
        focal_length: f64,
        aperture: f64,
    ) -> RealisticCamera {
        // Calculates orthonormal basis vectors for the camera
        let w = (position - look_at).unit();
        let u = Vector::cross(&up, &w).unit();
        let v = Vector::cross(&w, &u);

        // Calculates vector to the viewing plane
        let to_view_plane = -w * focal_length;

        // Constructs camera struct
        RealisticCamera {
            fov: fov.to_radians(),
            u,
            v,
            position,
            to_view_plane,
            focal_length,
            lens_radius: aperture / 2.0,
            pixels: None,
            samples: 0,
        }
    }

    /// Samples a ray from the camera to a specified pixel in an image.
    ///
    /// # Arguments
    ///
    /// * `px`: X position of the pixel.
    /// * `py`: Y position of the pixel.
    /// * `width`: Width of the image.
    /// * `height`: Height of the image.
    /// * `vw`: Width of the viewport.
    /// * `vh`: Height of the viewport.
    pub fn sample_pixel_ray(
        &self,
        px: i32,
        py: i32,
        width: i32,
        height: i32,
        vw: f64,
        vh: f64,
    ) -> Ray {
        // Calculates the position of a random point within the pixel as an offset from the image
        // plane's centre in the range [-1, 1], creating an antialiasing effect
        let fx = (vw * (px as f64 + random_float()) / width as f64) - (vw / 2.);
        let fy = (vh / 2.) - (vh * (py as f64 + random_float()) / height as f64);

        // Calculates depth of field ray offset based on the camera's lens radius
        let (rx, ry) = random_in_unit_disk();
        let offset = self.lens_radius * (self.u * rx + self.v * ry);

        // Calculates direction of the outgoing ray
        let direction = fx * self.u + fy * self.v + self.to_view_plane - offset;

        // Constructs ray to be traced through the scene
        Ray::new(self.position + offset, direction.unit())
    }

    /// Sets the starting pixels for rendering, if continuing from an existing PNG file.
    pub fn set_pixels(&mut self, pixels: Vec<[f64; 3]>) {
        self.pixels = Some(pixels);
    }

    /// Sets the starting samples for rendering, if continuing from an existing PNG file.
    pub fn set_samples(&mut self, samples: usize) {
        self.samples = samples;
    }
}

impl CameraTrait for RealisticCamera {
    fn render(self, scene: Scene, fb: &mut FrameBuffer) {
        // Gets the width and height of the FrameBuffer and viewport
        let (w, h) = (fb.get_width(), fb.get_height());
        let vh = 2.0 * self.focal_length * (0.5 * self.fov).tan();
        let vw = vh * (w as f64 / h as f64);

        // Creates vector of all output pixel values, using existing values if possible
        let n_pixels = w * h;
        let mut pixels = if self.pixels.is_none() {
            vec![[0.0; 3]; n_pixels as usize]
        } else {
            self.pixels.as_ref().unwrap().clone()
        };

        // Creates GUI for rendering progressive camera output
        let (gui, state) = RayTracerGUI::new(w as usize, h as usize);
        let state_clone = Arc::clone(&state);

        // Sender and receiver for passing pixel values out of the rendering thread once finished
        let (tx, rx) = mpsc::channel();

        // Spawns thread for rendering
        thread::spawn(move || {
            // Sets the starting sample count (will be 0 for new images)
            let mut n = self.samples;
            state.lock().unwrap().samples_completed = n;

            // Starts an indefinite loop, sampling one ray per pixel each iteration
            loop {
                // Stops any more samples from being taken while the renderer is paused
                if state.lock().unwrap().is_paused {
                    if state.lock().unwrap().is_stopping {
                        break;
                    }
                    continue;
                }

                // Increments number of samples taken for online mean calculation
                n += 1;

                // Loops through and samples a ray from each pixel
                pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                    // Calculates position of the current pixel
                    let y = i as i32 / w;
                    let x = i as i32 % w;

                    // Traces the pixel's ray through the scene and saves the recorded colour
                    let ray = self.sample_pixel_ray(x, y, w, h, vw, vh);
                    let colour = scene.camera_trace(&ray);

                    // Updates current average colour for the pixel across all samples
                    *pixel = [
                        pixel[0] + (colour.r - pixel[0]) / n as f64,
                        pixel[1] + (colour.g - pixel[1]) / n as f64,
                        pixel[2] + (colour.b - pixel[2]) / n as f64,
                    ];

                    // Updates the GUI image data with the new pixel value
                    let r_u8 = (255.0 * linear_to_srgb(pixel[0])) as u8;
                    let g_u8 = (255.0 * linear_to_srgb(pixel[1])) as u8;
                    let b_u8 = (255.0 * linear_to_srgb(pixel[2])) as u8;

                    state.lock().unwrap().image_data.set_pixel(
                        x as usize,
                        y as usize,
                        r_u8,
                        g_u8,
                        b_u8,
                    );
                });

                // Updates the number of samples completed on the GUI
                let mut _state = state.lock().unwrap();
                _state.samples_completed = n;

                // Checks if the renderer has been stopped
                if _state.is_stopping {
                    break;
                }
            }

            // Sets the renderer state to completed, and sends the pixel values out of the thread
            let mut _state = state.lock().unwrap();
            _state.is_stopping = false;
            _state.is_completed = true;
            tx.send((pixels, n)).unwrap();
        });

        // Runs the GUI
        gui.run().unwrap();

        // Returns immediately if the program was quit without stopping
        if !state_clone.lock().unwrap().is_completed {
            return;
        }

        // Gets pixels and sample count from the rendering thread
        let (pixels, samples) = rx.recv().unwrap();

        // Writes recorded pixel information to the FrameBuffer
        fb.set_samples_metadata(samples);
        for y in 0..h {
            for x in 0..w {
                let pixel = pixels[(y * w + x) as usize];
                fb.plot_pixel(x, y, pixel[0], pixel[1], pixel[2]);
            }
        }
    }
}

/// Implements loading a `RealisticCamera` from a YAML file.
impl FromYaml for RealisticCamera {
    fn from_yaml(yaml: &Yaml) -> Result<RealisticCamera, YamlPropertyError> {
        // Parses properties for the camera
        let fov = parse_float(yaml, "fov")?;
        let position = parse_struct(yaml, "position")?;
        let look_at = parse_struct(yaml, "look_at")?;
        let up = parse_struct(yaml, "up")?;
        let focal_length = parse_float(yaml, "focal_length")?;
        let aperture = parse_float(yaml, "aperture")?;

        // Returns the new camera instance
        Ok(RealisticCamera::new(
            fov,
            position,
            look_at,
            up,
            focal_length,
            aperture,
        ))
    }
}
