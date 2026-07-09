use crate::utils::gui::GUIBuffer;
use std::sync::{Arc, Mutex};

/// Struct to represent the parallel-safe mutable state of the GUI
pub struct GUIState {
    pub image_data: GUIBuffer,
    pub samples_completed: usize,
    pub is_stopping: bool,
    pub is_paused: bool,
    pub is_completed: bool,
}

impl GUIState {
    /// Creates a new default GUI state from a given image width and height.
    pub fn new(width: usize, height: usize) -> Arc<Mutex<GUIState>> {
        let image_data = GUIBuffer::new(width, height);
        let samples_completed = 0;
        let is_stopping = false;
        let is_paused = false;
        let is_completed = false;

        Arc::new(Mutex::new(GUIState {
            image_data,
            samples_completed,
            is_stopping,
            is_paused,
            is_completed,
        }))
    }
}
