use crate::core::{Hit, Ray};
use crate::objects::{Object, ObjectTrait};
use crate::utils::kd_tree::{Event, EventPlane, EventType, KDTreeNode, KDTreeNodeTrait};
use crate::utils::yaml::{parse_struct_array, parse_transforms, FromYaml, YamlPropertyError};
use crate::utils::{Transform, AABB};
use std::time::Instant;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Bounding object using a KDTree to partition the space and allow efficient intersection with
/// a list of objects.
///
/// Construction code based on the paper: On building fast kd-Trees for Ray Tracing, and on doing that in O(N log N) by I. Wald and V. Havran.
/// Paper can be found at: https://www.irisa.fr/prive/kadi/Sujets_CTR/kadi/Kadi_sujet2_article_Kdtree.pdf
pub struct KDTree {
    objects: Vec<Object>, // List of objects contained within the tree
    root: KDTreeNode,     // Root node of the tree

    bounding_box: AABB,   // Bounding box of the tree
    transform: Transform, // Transform of the tree
}

impl KDTree {
    /// Constructs a new KDTree from a list of objects.
    pub fn new(objects: Vec<Object>) -> KDTree {
        // Tracks start time of construction
        println!("Creating KDTree for {} objects...", objects.len());
        let start_time = Instant::now();

        // Computes the combined bounding box of all objects in the tree
        let bounding_box = objects
            .iter()
            .fold(AABB::empty(), |b, o| b.union(&o.get_bounding_box()));
        let object_indices = (0..objects.len()).collect();

        // Creates the sorted event list for the source objects
        let event_list = Self::create_event_list(&objects);

        // Starts recursive construction of the tree nodes
        let root = KDTreeNode::build(&objects, object_indices, bounding_box, event_list);

        // Displays construction time
        println!(
            "KDTree construction completed in: {:?}",
            start_time.elapsed()
        );

        KDTree {
            objects,
            root,
            bounding_box,
            transform: Transform::identity(),
        }
    }

    /// Creates a list containing all events from all dimensions.
    pub fn create_event_list(objects: &Vec<Object>) -> Vec<Event> {
        // Initialise event list, with a maximum capacity of two events per object per axis
        let mut event_list = Vec::with_capacity(objects.len() * 6);

        // Helper function for adding events to the event list
        let mut add_event = |index: usize, axis: u8, position: f64, event_type: EventType| {
            event_list.push(Event {
                object_index: index,
                plane: EventPlane { axis, position },
                event_type,
            })
        };

        // Iterate over each axis
        for axis in 0..3 {
            // Iterate over all objects and their indices
            for (index, object) in objects.iter().enumerate() {
                // Gets the object's bounding box and its min and max positions for the current axis
                let object_box = object.get_bounding_box();
                let min_pos = object_box.get_min(axis);
                let max_pos = object_box.get_max(axis);

                // Determines which type of event(s) to add to the list
                if min_pos == max_pos {
                    add_event(index, axis, min_pos, EventType::Planar);
                } else {
                    add_event(index, axis, min_pos, EventType::Start);
                    add_event(index, axis, max_pos, EventType::End);
                }
            }
        }

        // Sorts the event list to prepare for construction
        event_list.sort();
        event_list
    }
}

impl ObjectTrait for KDTree {
    fn intersection(&self, ray: &Ray) -> Vec<Hit<'_>> {
        // Transforms ray into the tree's object space
        let ray = ray.to_object_space(&self.transform);

        // Intersects the transformed ray with the tree
        let mut hits = self.root.intersection(&ray, &self.objects);

        // Transforms the hits back into world space
        hits.iter_mut().for_each(|h| h.transform(&self.transform));

        hits
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.bounding_box = self.bounding_box.transform(&transform);
        self.transform = transform * self.transform;
        self.transform.calculate_inverse();
    }

    fn get_bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

/// Implements loading a `KDTree` from a YAML file.
impl FromYaml for KDTree {
    fn from_yaml(yaml: &Yaml) -> Result<KDTree, YamlPropertyError> {
        // Parses the objects and uses them to create the KDTree instance
        let objects = parse_struct_array(yaml, "objects")?;
        let mut tree = KDTree::new(objects);

        // Applies any present transforms to the tree
        let transform = parse_transforms(yaml)?;
        tree.apply_transform(transform);

        Ok(tree)
    }
}
