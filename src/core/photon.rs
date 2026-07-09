use crate::drawing::Colour;
use crate::utils::{Vector, Vertex};
use std::cmp::Ordering;

#[derive(Debug)]
/// Photon struct used for photon mapping.
pub struct Photon {
    pub position: Vertex,  // Position of the photon
    pub direction: Vector, // Incident direction of the photon
    pub power: Colour,     // Power of the photon
}

impl Photon {
    /// Creates a new `Photon` instance with a given position, incident direction, and power.
    pub fn new(position: Vertex, direction: Vector, power: Colour) -> Photon {
        Photon {
            position,
            direction,
            power,
        }
    }

    /// Scales the photon's power by a given float.
    pub fn scale_power(&mut self, scale: f64) {
        self.power *= scale
    }
}

/// Trait implementations to allow photons to be stored in a max heap for the PhotonMap `find` operation.
/// Considers two photons as always equal.
impl Eq for Photon {}

impl PartialEq for Photon {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl PartialOrd for Photon {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for Photon {
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}
