use crate::core::{Hit, Ray};
use crate::objects::{Object, ObjectTrait};
use crate::utils::kd_tree::{Event, EventPlane, EventType, KDTreeLeaf, KDTreeSplit, ObjectSide};
use crate::utils::{merge, AABB};
use enum_dispatch::enum_dispatch;
use std::collections::HashMap;

#[enum_dispatch]
#[derive(Debug)]
/// Enum to represent both possible types of nodes in a KDTree.
pub enum KDTreeNode {
    KDTreeSplit,
    KDTreeLeaf,
}

impl KDTreeNode {
    /// Cost of traversing the tree (for SAH calculation).
    const TRAVERSAL_COST: f64 = 1.;

    /// Cost of intersecting an object (for SAH calculation).
    const INTERSECT_COST: f64 = 2.;

    /// Builds a node for a KDTree, recursing if a split occurs.
    ///
    /// # Arguments
    ///
    /// * `src_objects`: All source objects that make up the tree.
    /// * `current_objects`: Indices of the objects currently represented by this node.
    /// * `bounding_box`: The bounding box containing the current objects.
    /// * `event_list`: The event list for the current objects.
    pub fn build(
        src_objects: &Vec<Object>,
        current_objects: Vec<usize>,
        bounding_box: AABB,
        event_list: Vec<Event>,
    ) -> KDTreeNode {
        // Finds the best split plane and its associated cost
        let object_count = current_objects.len() as i32;
        let (best_cost, best_plane, left_split) =
            Self::find_plane(object_count, &bounding_box, &event_list);

        // Terminates tree construction if only one object remains or the cost of splitting is too high
        if best_cost > object_count as f64 * Self::INTERSECT_COST || object_count == 1 {
            return KDTreeLeaf::new(current_objects, bounding_box).into();
        }

        // Classifies and splits objects and events based on the best plane
        let (left, right, elo, ero, overlaps) =
            Self::classify_and_split(current_objects, event_list, best_plane, left_split);

        // Splits the current volume into left and right bounding boxes based on the best plane
        let (left_box, right_box) = bounding_box.split(best_plane.into());

        // Generates new events for objects overlapping the best plane
        let mut ebl = Vec::with_capacity(overlaps.len() * 6);
        let mut ebr = Vec::with_capacity(overlaps.len() * 6);
        for i in overlaps {
            // Gets the object's bounding box on both sides of the split plane
            let bl = src_objects[i].get_bounding_box().intersect(&left_box);
            let br = src_objects[i].get_bounding_box().intersect(&right_box);

            // Adds new events for each axis
            for axis in 0..3 {
                Self::add_new_events(&mut ebl, i, axis, &bl);
                Self::add_new_events(&mut ebr, i, axis, &br);
            }
        }

        // Sorts newly created events
        ebl.sort();
        ebr.sort();

        // Merges four event strains into combined (sorted) left and right event lists
        let el = merge(elo, ebl);
        let er = merge(ero, ebr);

        // Recursively builds new nodes for the left and right subtree
        let (left, right) = rayon::join(
            || KDTreeNode::build(src_objects, left, left_box, el).into(),
            || KDTreeNode::build(src_objects, right, right_box, er).into(),
        );
        KDTreeSplit::new(left, right, bounding_box, best_plane).into()
    }

    /// Adds new events for a given object, axis, and bounding box to an event list.
    fn add_new_events(
        event_list: &mut Vec<Event>,
        object_index: usize,
        axis: u8,
        bounding_box: &AABB,
    ) {
        // Calculates min and max positions of the bounding box for the axis
        let min_pos = bounding_box.get_min(axis);
        let max_pos = bounding_box.get_max(axis);

        // Helper function for adding events to the event list
        let mut add_event = |index: usize, axis: u8, position: f64, event_type: EventType| {
            event_list.push(Event {
                object_index: index,
                plane: EventPlane { axis, position },
                event_type,
            })
        };

        // Catches planar case and returns
        if min_pos == max_pos {
            add_event(object_index, axis, min_pos, EventType::Planar);
            return;
        }

        // Adds regular start and end events
        add_event(object_index, axis, min_pos, EventType::Start);
        add_event(object_index, axis, max_pos, EventType::End);
    }

    /// Classifies objects and events into being left or right of the best plane, and splits them
    /// into corresponding lists.
    ///
    /// # Arguments
    ///
    /// * `objects`: Indices of the objects to be classified.
    /// * `event_list`: Events to be classified.
    /// * `best_plane`: The best split plane for the space.
    /// * `left_split`: Whether to classify objects planar to the best plane into the left set.
    ///
    /// returns: (left, right, elo, ero, overlaps)
    /// * `left`: Objects to the left of or overlapping the best plane.
    /// * `right`: Objects to the right of or overlapping the best plane.
    /// * `elo`: Events to the left of the best plane.
    /// * `ero`: Events to the right of the best plane.
    /// * `overlaps`: Indices of objects that overlap the best plane.
    fn classify_and_split(
        objects: Vec<usize>,
        event_list: Vec<Event>,
        best_plane: EventPlane,
        left_split: bool,
    ) -> (Vec<usize>, Vec<usize>, Vec<Event>, Vec<Event>, Vec<usize>) {
        // Creates a set to keep track of which objects have been classified
        let mut classifications = HashMap::with_capacity(objects.len());

        // Initialises vectors for storing the objects left and right of the split
        let mut left_objects = Vec::new();
        let mut right_objects = Vec::new();

        // Helper function for adding to an object list
        let mut add = |obj_list: &mut Vec<usize>, index: usize, side: ObjectSide| {
            obj_list.push(index);
            classifications.insert(index, side);
        };

        // Iterate through the events to classify objects based on them
        for event in event_list.iter() {
            match event.event_type {
                // Classify as left if the object's end point is to the left of the best plane
                EventType::End
                    if event.plane.axis == best_plane.axis
                        && event.plane.position <= best_plane.position =>
                {
                    add(&mut left_objects, event.object_index, ObjectSide::Left)
                }

                // Classify as right if the object's start point is to the right of the best plane
                EventType::Start
                    if event.plane.axis == best_plane.axis
                        && event.plane.position >= best_plane.position =>
                {
                    add(&mut right_objects, event.object_index, ObjectSide::Right)
                }

                // Classify objects that lie parallel to the best plane
                EventType::Planar if event.plane.axis == best_plane.axis => {
                    // Planar objects to the left of the best plane
                    if event.plane.position < best_plane.position
                        || (event.plane.position == best_plane.position && left_split)
                    {
                        add(&mut left_objects, event.object_index, ObjectSide::Left)
                    }

                    // Planar objects to the right of the best plane
                    if event.plane.position > best_plane.position
                        || (event.plane.position == best_plane.position && !left_split)
                    {
                        add(&mut right_objects, event.object_index, ObjectSide::Right)
                    }
                }

                // Ignore other cases
                _ => {}
            }
        }

        // Add objects that weren't classified to both sides, and to the overlaps list
        let mut overlaps = Vec::with_capacity(objects.len() - classifications.len());
        for object in objects {
            if !classifications.contains_key(&object) {
                left_objects.push(object);
                right_objects.push(object);
                overlaps.push(object);
            }
        }

        // Splices events for objects that do not overlap into two sets
        let mut elo = Vec::with_capacity(left_objects.len());
        let mut ero = Vec::with_capacity(right_objects.len());
        for event in event_list {
            if let Some(classification) = classifications.get(&event.object_index) {
                match classification {
                    ObjectSide::Left => elo.push(event),
                    ObjectSide::Right => ero.push(event),
                }
            }
        }

        (left_objects, right_objects, elo, ero, overlaps)
    }

    /// Calculates the plane cost for the surface area heuristic.
    ///
    /// # Arguments
    ///
    /// * `pl`: Proportion of the surface area on the left of the plane.
    /// * `pr`: Proportion of the surface area on the right of the plane.
    /// * `nl`: Number of objects on the left of the plane.
    /// * `nr`: Number of objects on the right of the plane.
    fn plane_cost(pl: f64, pr: f64, nl: i32, nr: i32) -> f64 {
        let lambda = if nl == 0 || nr == 0 { 0.8 } else { 1. };
        lambda * (Self::TRAVERSAL_COST + Self::INTERSECT_COST * (pl * nl as f64 + pr * nr as f64))
    }

    /// Calculates the surface area heuristic cost for a given event plane
    ///
    /// # Arguments
    ///
    /// * `p`: Event plane to calculate the cost for.
    /// * `v`: Bounding volume to be split.
    /// * `nl`: Number of objects to the left of the plane.
    /// * `nr`: Number of objects to the right of the plane.
    /// * `np`: Number of objects lying on the plane.
    fn surface_area_heuristic(p: EventPlane, v: &AABB, nl: i32, nr: i32, np: i32) -> (f64, bool) {
        // Splits the bounding volume and calculates its surface area
        let (vl, vr) = v.split(p.into());
        let v_area_recip = 1. / v.surface_area();

        // Calculates the surface area proportion of each side of the split
        let pl = vl.surface_area() * v_area_recip;
        let pr = vr.surface_area() * v_area_recip;

        // Eliminates cases where the plane lies on the edge of the bounding volume
        if vl.get_min(p.axis) == vl.get_max(p.axis) || vr.get_min(p.axis) == vr.get_max(p.axis) {
            return (f64::MAX, false);
        }

        // Calculates the cost for both possible combinations of objects
        let cl = Self::plane_cost(pl, pr, nl + np, nr);
        let cr = Self::plane_cost(pl, pr, nl, nr + np);

        // Returns the best cost
        (cl.min(cr), cl < cr)
    }

    /// Uses the surface area heuristic to find the best plane to split a bounding box.
    ///
    /// # Arguments
    ///
    /// * `object_count`: The number of objects contained within the bounding box.
    /// * `bounding_box`: The bounding box to be split.
    /// * `event_list`: The sorted events list for sweeping candidate planes across the bounding box.
    ///
    /// returns: (f64, EventPlane, bool)
    /// * `best_cost`: The cost (calculated using the surface area heuristic) of the best plane.
    /// * `best_plane`: The best plane found for splitting the bounding box.
    /// * `left_split`: Whether to classify objects planar to the best plane into the left set.
    fn find_plane(
        object_count: i32,
        bounding_box: &AABB,
        event_list: &Vec<Event>,
    ) -> (f64, EventPlane, bool) {
        // Variable to store the current best plane
        let mut best = (
            f64::MAX,
            EventPlane {
                axis: 0,
                position: 0.0,
            },
            false,
        );

        // Number of objects left and right of the current plane position in each axis
        let mut nl = [0, 0, 0];
        let mut nr = [object_count, object_count, object_count];

        // Sweeps a plane over all split candidates
        let mut i = 0;
        while i < event_list.len() {
            // Gets the current plane to evaluate
            let plane = event_list[i].plane;
            let axis = plane.axis as usize;

            // Counts number of objects starting, ending, or lying in the candidate plane
            let (mut ps, mut pe, mut pl) = (0, 0, 0);
            while i < event_list.len() && event_list[i].plane == plane {
                match event_list[i].event_type {
                    EventType::Start => ps += 1,
                    EventType::Planar => pl += 1,
                    EventType::End => pe += 1,
                }
                i += 1
            }

            // Removes objects ending on or lying on the plane from the right counter
            nr[axis] -= pl + pe;

            // Calculates the cost of splitting at the candidate plane
            let (c, left_split) =
                Self::surface_area_heuristic(plane, bounding_box, nl[axis], nr[axis], pl);

            // Updates the best candidate if a lower cost has been found
            if c < best.0 {
                best = (c, plane, left_split);
            }

            // Adds counts of objects starting on or lying on the plane to the left counter
            nl[axis] += ps + pl;
        }

        best
    }
}

#[enum_dispatch(KDTreeNode)]
/// Trait implemented by both types of KD-Tree nodes.
pub trait KDTreeNodeTrait {
    /// Returns all points of intersection, sorted by `t`, if the given ray intersects the node's subtree.
    fn intersection<'a>(&self, ray: &Ray, objects: &'a Vec<Object>) -> Vec<Hit<'a>>;

    /// Gets the bounding box of the node.
    fn get_bounding_box(&self) -> &AABB;
}
