use crate::core::Hit;
use crate::drawing::{srgb_to_linear, Colour};
use indicatif::{ProgressBar, ProgressStyle};
use png::Decoder;
use std::fs::File;
use yaml_rust::{Yaml, YamlLoader};

/// Gets a progress bar with this project's default styling.
///
/// # Arguments
///
/// * `len`: Length of the progress bar.
pub fn get_default_progress_bar(len: u64) -> ProgressBar {
    let progress_bar_style = ProgressStyle::with_template(
        "{spinner:.green} \
        [{elapsed_precise}] \
        [{wide_bar:.cyan/blue}] \
        {human_pos}/{human_len} \
        ({eta})",
    )
    .unwrap()
    .progress_chars("#>-");

    let progress_bar = ProgressBar::new(len);
    progress_bar.set_style(progress_bar_style);

    progress_bar
}

/// Filters a list of  hits, returning the closest positive hit.
///
/// # Arguments
///
/// * `hits`: A list of hits, ordered by ascending `t` value.
pub fn select_first(hits: Vec<Hit>) -> Option<Hit> {
    hits.into_iter().find(|hit| hit.t >= 0.)
}

/// Returns a random float in the range [0, 1].
pub fn random_float() -> f64 {
    rand::random()
}

/// Returns a random float in the range [min, max].
pub fn random_range(min: f64, max: f64) -> f64 {
    min + (max - min) * random_float()
}

/// Returns a random point in a 2D unit disk, given as an (x, y) tuple.
pub fn random_in_unit_disk() -> (f64, f64) {
    // Uses rejection sampling to find a valid random point
    loop {
        // Creates random x and y coordinates within a square
        let x = random_range(-1.0, 1.0);
        let y = random_range(-1.0, 1.0);

        // Return the point if it lies within the unit disk
        if x * x + y * y <= 1. {
            return (x, y);
        }
    }
}

/// Normalises a list of colour coefficients to make them energy conserving.
pub fn normalise_colour_coefficients(colours: &mut Vec<Colour>) {
    // Gets the maximum channel value of the sum of the input colours
    let scale = colours.iter().fold(Colour::black(), |a, b| a + *b).max();

    // Normalises the colours to ensure energy conservation
    if scale > 1.0 {
        for colour in colours {
            *colour /= scale
        }
    }
}

/// Merges two sorted lists together in O(n).
pub fn merge<T: PartialOrd + Clone>(left: Vec<T>, right: Vec<T>) -> Vec<T> {
    let nl = left.len();
    let nr = right.len();

    let mut merged = Vec::with_capacity(left.len() + right.len());
    let mut i = 0;
    let mut j = 0;

    // Merge by moving the lower element from left and right into merged at each step
    while i < nl && j < nr {
        if left[i] <= right[j] {
            merged.push(left[i].clone());
            i += 1;
        } else {
            merged.push(right[j].clone());
            j += 1;
        }
    }

    // Handle remaining elements in the left list
    while i < nl {
        merged.push(left[i].clone());
        i += 1;
    }

    // Handle remaining elements in the right list
    while j < nr {
        merged.push(right[j].clone());
        j += 1;
    }

    merged
}

/// Reads in and returns the contents of the given file as a `String`.
pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(filename).expect(format!("Unable to open file {}.", filename).as_str())
}

/// Reads in a YAML file and returns its contents as a `Yaml` instance.
pub fn read_yaml(filename: &str) -> Yaml {
    // Reads in YAML documents from the file
    let contents = read_file(filename);
    let docs = YamlLoader::load_from_str(&contents)
        .expect("Error while scanning YAML file, ensure correct YAML formatting is used.");

    // Returns the first document loaded
    match docs.get(0) {
        None => panic!("Given file does not contain a valid YAML document."),
        Some(doc) => doc.clone(),
    }
}

/// Adds a field to a `Yaml` instance (assuming the instance is of type Hash).
pub fn add_yaml_field(yaml: Yaml, key: &str, field: &str) -> Yaml {
    // Attempts to convert `Yaml` instance into a Hash
    let mut yaml_hash = yaml
        .into_hash()
        .expect("Attempted to add a field to an invalid Yaml instance.");

    // Inserts the new string and returns the updated `Yaml` instance
    yaml_hash.insert(
        Yaml::String(key.to_string()),
        Yaml::String(field.to_string()),
    );
    Yaml::Hash(yaml_hash)
}

/// Normalises file paths to Unix style for cross-platform consistency.
pub fn normalise_filepath(path: &str) -> String {
    let mut normalised = path.replace('\\', "/");

    if normalised.starts_with("./") {
        normalised = normalised[2..].to_string();
    }

    normalised
}

/// Reads in a PNG file for continued rendering and returns its contents as float pixel values,
/// along with its expected metadata.
pub fn read_png(filename: &str) -> (Vec<[f64; 3]>, usize, String) {
    let file = File::open(filename).expect("Unable to open file");
    let decoder = Decoder::new(file);

    // Creates reader for the PNG file
    let mut reader = decoder.read_info().expect("Failed to read PNG info");

    // Reads PNG metadata
    let mut samples = 0;
    let mut scene_filename = String::new();
    for chunk in reader.info().uncompressed_latin1_text.iter() {
        match chunk.keyword.as_str() {
            "samples" => samples = chunk.text.parse().unwrap(),
            "scene_filename" => scene_filename = chunk.text.to_string(),
            _ => {}
        }
    }

    // Reads image data
    let mut data = vec![0; reader.output_buffer_size()];
    reader
        .next_frame(&mut data)
        .expect("Failed to read PNG data.");

    // Converts image into pixel array
    let mut image_data = Vec::new();
    let num_pixels = (reader.info().width * reader.info().height) as usize;

    match reader.info().color_type {
        png::ColorType::Rgb => {
            // For RGB (3 bytes per pixel)
            for i in 0..num_pixels {
                let r = data[i * 3 + 0] as f64 / 255.0;
                let g = data[i * 3 + 1] as f64 / 255.0;
                let b = data[i * 3 + 2] as f64 / 255.0;
                image_data.push([r, g, b]);
            }
        }
        png::ColorType::Rgba => {
            // For RGBA (4 bytes per pixel)
            for i in 0..num_pixels {
                let r = data[i * 4 + 0] as f64 / 255.0;
                let g = data[i * 4 + 1] as f64 / 255.0;
                let b = data[i * 4 + 2] as f64 / 255.0;
                image_data.push([r, g, b]);
            }
        }
        _ => {
            panic!("Unsupported color type. Only RGB or RGBA are supported.");
        }
    }

    // Applies sRGB decoding
    for pixel in &mut image_data {
        pixel[0] = srgb_to_linear(pixel[0]);
        pixel[1] = srgb_to_linear(pixel[1]);
        pixel[2] = srgb_to_linear(pixel[2]);
    }

    (image_data, samples, scene_filename)
}
