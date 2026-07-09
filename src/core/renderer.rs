use crate::cameras::{Camera, CameraTrait};
use crate::core::Scene;
use crate::drawing::FrameBuffer;
use crate::utils::yaml::{
    parse_int, parse_string, parse_struct, FromYaml, YamlPropertyError,
};
use crate::utils::{add_yaml_field, read_png, read_yaml};
use crate::{HEIGHT, WIDTH};
use yaml_rust::Yaml;

/// Struct to wrap every element of rendering, used primarily for easily loading both scenes
/// and cameras from a file into a single object.
pub struct Renderer {
    scene: Scene,   // Scene to be rendered
    camera: Camera, // Camera to render the scene with

    // Additional frame buffer settings
    width: i32,
    height: i32,
    scene_filename: String,
}

impl Renderer {
    /// Creates a new renderer with a given scene, camera, and frame buffer settings.
    pub fn new(
        scene: Scene,
        camera: Camera,
        width: i32,
        height: i32,
        scene_filename: String,
    ) -> Renderer {
        Renderer {
            scene,
            camera,
            width,
            height,
            scene_filename,
        }
    }

    /// Creates a new renderer from the path to a previously output PNG file.
    pub fn from_png(filename: &str) -> Result<Renderer, YamlPropertyError> {
        // Reads in pixel values and relevant metadata from the PNG
        let (pixels, samples, scene_filename) = read_png(filename);

        // Loads the renderer from the YAML file referenced in metadata
        let yaml_contents = add_yaml_field(
            read_yaml(scene_filename.as_str()),
            "scene_filename",
            scene_filename.as_str(),
        );
        let mut renderer = Renderer::from_yaml(&yaml_contents)?;

        // Sets the camera's values to the stored PNG values if using a realistic camera
        if let Camera::RealisticCamera(camera) = &mut renderer.camera {
            camera.set_pixels(pixels);
            camera.set_samples(samples);
        }

        // Returns the constructed renderer
        Ok(renderer)
    }

    /// Renders the scene and stores it as a PNG file with the given filename.
    pub fn render_to_file(self, filename: &str) {
        self.render_to_buffer().write_to_file(filename);
    }

    // Renders the scene to a FrameBuffer and returns it.
    pub fn render_to_buffer(self) -> FrameBuffer {
        let mut fb = FrameBuffer::new(self.width, self.height, self.scene_filename);
        self.camera.render(self.scene, &mut fb);
        fb
    }
}

/// Implements loading a `Renderer` from a YAML file.
impl FromYaml for Renderer {
    fn from_yaml(yaml: &Yaml) -> Result<Renderer, YamlPropertyError> {
        // Parses camera and scene objects
        let camera = parse_struct(yaml, "camera")?;
        let scene = parse_struct(yaml, "scene")?;

        // Parses optional frame buffer arguments, using defaults where they are not given
        let scene_filename = parse_string(yaml, "scene_filename")?;
        let width = parse_int(yaml, "width").unwrap_or(WIDTH) as i32;
        let height = parse_int(yaml, "height").unwrap_or(HEIGHT) as i32;

        // Returns constructed renderer object
        Ok(Renderer::new(
            scene,
            camera,
            width,
            height,
            scene_filename,
        ))
    }
}
