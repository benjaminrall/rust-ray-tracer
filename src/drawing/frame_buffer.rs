use crate::drawing::linear_to_srgb;
use crate::drawing::Pixel;
use image::{ImageBuffer, Rgb};
use png::{Decoder, Encoder};
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use crate::utils::normalise_filepath;

/// The FrameBuffer class creates a buffer of RGB values that can be written to a PNG file.
pub struct FrameBuffer {
    // General frame buffer attributes
    framebuffer: Vec<Pixel>,
    width: i32,
    height: i32,

    // Metadata for the output image
    samples_metadata: usize,
    scene_filename_metadata: String,
}

impl FrameBuffer {
    /// Creates a new, empty `FrameBuffer` with a given width, height, and scene filename.
    pub fn new(w: i32, h: i32, scene_filename: String) -> FrameBuffer {
        let framebuffer = vec![Pixel::new(0., 0., 0.); (w * h) as usize];
        FrameBuffer {
            width: w,
            height: h,
            framebuffer,
            samples_metadata: 1,
            scene_filename_metadata: scene_filename,
        }
    }

    /// Gets the width of the `FrameBuffer`
    pub fn get_width(&self) -> i32 {
        self.width
    }

    /// Gets the height of the `FrameBuffer`
    pub fn get_height(&self) -> i32 {
        self.height
    }

    /// Returns true if the given (x, y) coordinates are within the bounds of the `FrameBuffer`.
    fn valid_pixel(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Returns the index in the `FrameBuffer` of given (x, y) coordinates.
    fn get_index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Plots a pixel's RGB value to the `FrameBuffer`.
    pub fn plot_pixel(&mut self, x: i32, y: i32, red: f64, green: f64, blue: f64) {
        // Checks that the given coordinates are within a valid range
        if !self.valid_pixel(x, y) {
            return;
        }

        // Updates RGB values
        let i = self.get_index(x, y);
        self.framebuffer[i].red = red;
        self.framebuffer[i].green = green;
        self.framebuffer[i].blue = blue;
    }

    /// Gets a reference to a pixel from the `FrameBuffer`.
    pub fn get_pixel(&self, x: i32, y: i32) -> Result<&Pixel, &'static str> {
        // Checks that the given coordinates are within a valid range
        if !self.valid_pixel(x, y) {
            return Err("Pixel coordinates out of range.");
        }

        // Gets and returns the pixel reference
        let i = self.get_index(x, y);
        Ok(&self.framebuffer[i])
    }

    /// Sets the samples metadata for the buffer.
    pub fn set_samples_metadata(&mut self, samples: usize) {
        self.samples_metadata = samples;
    }

    /// Writes the `FrameBuffer`'s pixel information to a PNG file.
    pub fn write_to_file(&self, filename: &str) {
        // Writes the data into an `ImageBuffer` to be saved as a PNG
        let mut buffer = ImageBuffer::new(self.width as u32, self.height as u32);
        buffer.par_enumerate_pixels_mut().for_each(|(x, y, pixel)| {
            let i = self.get_index(x as i32, y as i32);

            // Converts raw pixel float values into clamped sRGB u8s.
            let r = (255.0 * linear_to_srgb(self.framebuffer[i].red)) as u8;
            let g = (255.0 * linear_to_srgb(self.framebuffer[i].green)) as u8;
            let b = (255.0 * linear_to_srgb(self.framebuffer[i].blue)) as u8;

            // Stores the pixel to the image buffer
            *pixel = Rgb([r, g, b]);
        });

        // Writes the image data to the PNG file
        match buffer.save(filename) {
            Err(e) => eprintln!("Error writing to file: {}", e),
            Ok(()) => eprintln!("Image saved successfully."),
        }

        // Opens the saved PNG file in order to add metadata to it
        let file = File::open(filename).expect("Error opening saved file.");
        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader);
        let mut png_reader = decoder
            .read_info()
            .expect("Unable to read PNG info from saved file.");

        // Reads in the image pixel values
        let mut data = vec![0; png_reader.output_buffer_size()];
        png_reader
            .next_frame(&mut data)
            .expect("Unable to read PNG data from saved file.");

        // Overwrites the file with a new empty one
        let file = File::create(filename).expect("Error creating output file.");
        let writer = BufWriter::new(file);

        // Sets relevant colour and depth information about the new file
        let info = png_reader.info();
        let mut encoder = Encoder::new(writer, info.width, info.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);

        // Adds general metadata tags
        encoder
            .add_text_chunk(String::from("samples"), self.samples_metadata.to_string())
            .expect("Failed to add samples metadata to file.");
        encoder
            .add_text_chunk(
                String::from("scene_filename"),
                normalise_filepath(&self.scene_filename_metadata),
            )
            .expect("Failed to add samples metadata to file.");

        // Sets the sRGB metadata value
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);

        // Writes the metadata header to the file
        let mut png_writer = encoder.write_header().expect("Error writing PNG header.");

        // Writes pixel values back to the file
        png_writer
            .write_image_data(&data)
            .expect("Error writing PNG data.");

        println!("Updated image metadata successfully.")
    }
}
