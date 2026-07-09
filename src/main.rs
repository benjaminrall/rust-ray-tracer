use ray_tracer::core::Renderer;
use ray_tracer::utils::yaml::FromYaml;
use ray_tracer::utils::{add_yaml_field, read_yaml};

fn main() {
    // Gets command line arguments
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!(
            "Incorrect command line arguments provided!\n\
            Correct usage: cargo run --release <scene yaml file path | previously output png file> <optional output png file path>"
        );
    }

    // Accounts for both types of input, direct from YAML files and using a previously generated output
    if args[1].ends_with(".yaml") {
        // Gets the output file path
        let output_file = if args.len() == 3 {
            &args[2]
        } else {
            "output.png"
        };

        // Reads in the YAML file, adds in the scene filename, and then uses it to create and render the scene
        let yaml_contents = add_yaml_field(read_yaml(&args[1]), "scene_filename", &args[1]);
        match Renderer::from_yaml(&yaml_contents) {
            Ok(renderer) => renderer.render_to_file(output_file),
            Err(e) => panic!("Error loading renderer properties from file:\n{}", e),
        }
    } else if args[1].ends_with(".png") {
        // Gets the output file path
        let output_file = if args.len() == 3 {
            &args[2]
        } else {
            &args[1]
        };

        // Creates and renders the scene based on the PNG files stored metadata
        match Renderer::from_png(&args[1]) {
            Ok(renderer) => renderer.render_to_file(output_file),
            Err(e) => panic!("Error loading renderer properties from file:\n{}", e),
        }
    } else {
        panic!("Unsupported file type given.")
    }
}
