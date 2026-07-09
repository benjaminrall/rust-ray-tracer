use std::cmp::Ordering;

#[repr(u8)]
#[derive(Ord, Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
/// Enum to represent the possible types of events during KDTree construction.
pub enum EventType {
    Start = 2,  // + event
    Planar = 1, // | event
    End = 0,    // - event
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Enum to represent an axis-aligned plane at which splits can be made.
pub struct EventPlane {
    pub axis: u8,
    pub position: f64,
}

/// Implements conversion from an EventPlane into a tuple of its axis and position
impl Into<(u8, f64)> for EventPlane {
    fn into(self) -> (u8, f64) {
        (self.axis, self.position)
    }
}

#[derive(Debug, Copy, Clone)]
/// Struct to represent events for sweeping planes across a bounding box.
pub struct Event {
    pub object_index: usize,
    pub plane: EventPlane,
    pub event_type: EventType,
}

/// Implements equality comparison for events based on their plane and event type.
impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.plane == other.plane && self.event_type == other.event_type
    }
}
impl Eq for Event {}

/// Implements partial ordering for events based on their plane position and event type.
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implements total ordering for events based on their plane position and event type.
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.plane.position.partial_cmp(&other.plane.position) {
            Some(Ordering::Equal) | None => self.event_type.cmp(&other.event_type),
            Some(order) => order,
        }
    }
}
