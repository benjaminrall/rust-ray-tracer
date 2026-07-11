# Rust Ray Tracer

Fully-featured CPU ray tracer written in Rust, using Photon Mapping and Monte Carlo ray tracing techniques to generate photorealistic images.

## Table of Contents

- [Key Features](#key-features)
- [Gallery](#gallery)
- [Installation and Usage](#installation-and-usage)
- [YAML Scene Reference Guide](#yaml-scene-reference-guide)
- [Technical Details](#technical-details)
- [License](#license)

## Key Features

- **Global Illumination**: Implements advanced Photon Mapping (using dedicated global, caustic, and volumetric maps) for accurate indirect light transport and caustics.
- **Monte Carlo Techniques**: Uses Monte Carlo integration for accurate radiance estimates from the global photon map and to simulate glossy reflection and refraction.
- **Parallelised Operations**: Utilises the `rayon` crate to distribute ray tracing across all available CPU cores, maximising multi-threading performance.
- **KD-Tree Acceleration**: Custom KD-tree implementation optimises intersection testing for complex polygon meshes, using the Surface Area Heuristic (SAH) for highly efficient construction.
- **Advanced Geometry**: Supports a wide range of geometric primitives alongside Wavefront `.obj` meshes, Constructive Solid Geometry (CSG), and object instancing.
- **Physically Based Camera**: Features a thin-lens camera model to simulate realistic depth of field effects.
- **Live Rendering Preview**: Includes a thread-safe graphical user interface to monitor progressive rendering and sample accumulation.
- **YAML Scene Definition**: Streamlined scene design via a robust, custom YAML format.

## Gallery

## Installation and Usage

To run the ray tracer locally and generate your own images, you will need the standard Rust toolchain installed. Follow these steps:

1. **Clone the repository**:

    ```bash
    git clone https://github.com/benjaminrall/rust-ray-tracer.git
    cd rust-ray-tracer
    ```

2. **Define your scene**:

    Use the [YAML Scene Reference Guide](#yaml-scene-reference-guide) or refer to the provided [example scenes](./scenes) to construct the scene you wish to render.

3. **Run the application**:

    You can run the ray tracer in two ways using Cargo. 

    *   **Render a new scene:**

        Pass the path to your YAML file and an optional output path (defaults to `output.png`).
        ```bash
        cargo run --release -- your_scene.yaml your_output.png
        ```

    *   **Resume a render:**
        
        If using the `RealisticCamera`, you can continue rendering an image by passing the path to a previously generated PNG.

        ```bash
        cargo run --release -- your_image.png
        ```

        This requires the original YAML file to remain at its original path. By default, the input image will be overwritten with the new samples, but you can optionally also provide an output path to save to a new file instead.


## YAML Scene Reference Guide

## Technical Details

## License

This project is licensed under the **MIT License**. See the [`LICENSE`](./LICENSE) file for details.
