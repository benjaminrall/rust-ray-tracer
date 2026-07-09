use crate::drawing::Colour;
use crate::utils::gui::{GUIBuffer, GUIState};
use eframe::egui::{
    CentralPanel, ColorImage, Context, Layout, TextureHandle, Vec2, ViewportBuilder,
};
use eframe::{egui, App, Frame};
use std::sync::{Arc, Mutex};

/// Ray tracer GUI for viewing ongoing Realistic Camera renders.
pub struct RayTracerGUI {
    state: Arc<Mutex<GUIState>>,
    texture: Option<TextureHandle>,
    width: usize,
    height: usize,
}

impl RayTracerGUI {
    /// Constructs a new Ray Tracer GUI for an image with a given width and height.
    ///
    /// Returns the RayTracerGUI instance, along with clones of its internal state for
    /// modification and use.
    pub fn new(width: usize, height: usize) -> (RayTracerGUI, Arc<Mutex<GUIState>>) {
        // Creates the state for the GUI
        let state = GUIState::new(width, height);

        // Creates the GUI
        let gui = RayTracerGUI {
            state: state.clone(),
            texture: None,
            width,
            height,
        };

        (gui, state)
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        // Calculates width and height of the window
        let iw = (self.width + 16) as f32;
        let ih = (self.height + 32) as f32;

        // Create viewport for the GUI
        let viewport = ViewportBuilder::default()
            .with_title("Ray Tracer")
            .with_fullscreen(false)
            .with_min_inner_size(Vec2::new(iw, ih))
            .with_max_inner_size(Vec2::new(iw, ih))
            .with_inner_size(Vec2::new(iw, ih))
            .with_maximize_button(false);

        // Add viewport to the frame's options
        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        // Runs the GUI
        eframe::run_native("Ray Tracer", options, Box::new(|_cc| Ok(Box::new(self))))
    }
}

impl App for RayTracerGUI {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut state = self.state.lock().unwrap();

        // Create or update the texture for the GUI
        if self.texture.is_none() {
            self.texture = Some(ctx.load_texture(
                "output",
                ColorImage::from_rgb([self.width, self.height], state.image_data.get_ref()),
                egui::TextureOptions::LINEAR,
            ));
        } else if let Some(texture) = &mut self.texture {
            texture.set(
                ColorImage::from_rgb([self.width, self.height], state.image_data.get_ref()),
                Default::default(),
            )
        }

        // Creates the GUI display panel
        CentralPanel::default().show(ctx, |ui| {
            // Adds image to the display
            if let Some(texture) = &self.texture {
                ui.image(texture);
            }

            ui.horizontal(|ui| {
                // Adds sample text to the display
                let samples_completed = state.samples_completed;
                ui.label(format!("Samples Completed: {}", samples_completed));

                // Adds stop button to the display
                ui.with_layout(Layout::right_to_left(Default::default()), |ui| {
                    if state.is_completed {
                        ui.label("Ray Tracer Finished!");
                    } else if state.is_stopping {
                        state.is_paused = false;
                        ui.label("Stopping Ray Tracer...");
                    } else {
                        state.is_stopping = ui.button("Stop").clicked();

                        if state.is_paused {
                            state.is_paused = !ui.button("Unpause").clicked();
                        } else {
                            state.is_paused = ui.button("Pause").clicked();
                        }
                    }
                });
            })
        });

        // Redraws the display
        ctx.request_repaint();
    }
}
