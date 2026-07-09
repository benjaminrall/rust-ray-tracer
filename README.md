# CM30075 Ray Tracing Code

The Rust files provided in `src/` contain all necessary
code to run the ray tracer.

`final_output.png` is the final image output chosen for
the ray tracer, and its scene definition can be found
in `final_scene.yaml`. It consists of a shiny porcelain-like teapot
on a wooden table, with a glass of tea in the foreground, being entirely
lit by two candles (which are represented in the scene as constant density participating media).

To run the program, `rustup` must first be installed.
Then, to render the final output image, the following command must be run in the terminal:

```bash
cargo run --release final_scene.yaml final_output.png
```

This will compile and run the program. First, progress
bars for photon mapping will be displayed in the terminal.
Then, the ray tracing GUI will be shown. This contains
the current rendering progress, along with the number
of samples completed and a Pause and Stop button. Once
the scene has been rendered for a sufficient number of
samples, pressing the Stop button will finish the current
sample and then display that the render is complete.
Closing the GUI window after this point will save the output
file to `final_output.png`.
