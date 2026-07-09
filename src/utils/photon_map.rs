use crate::core::{Hit, Photon};
use crate::drawing::Colour;
use crate::utils::{FilterType, Vertex, AABB};
use crate::CONE_FILTER_K;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;
use std::time::Instant;

#[derive(Debug)]
/// Balanced kd-tree for storing Photons.
pub struct PhotonMap {
    pub tree: Vec<Photon>,
    pub splits: Vec<u8>,
}

impl PhotonMap {
    /// Creates a new PhotonMap from the given Photon array.
    pub fn new(mut photons: Vec<Photon>) -> PhotonMap {
        // Tracks start time of construction
        println!("Creating Photon Map for {} photons...", photons.len());
        let start_time = Instant::now();

        let mut splits = vec![0; photons.len()];

        // Builds the tree using the given photons
        Self::build(&mut photons, &mut splits);

        // Displays construction time
        println!(
            "Photon Map construction completed in: {:?}",
            start_time.elapsed()
        );

        PhotonMap {
            tree: photons,
            splits,
        }
    }

    /// Recursively builds a balanced kd-tree from a given set of photons.
    ///
    /// # Arguments
    ///
    /// * `photons`: Set of photons to build the kd-tree from.
    /// * `axis`: Axis to split along when creating the kd-tree.
    pub fn build(photons: &mut [Photon], splits: &mut [u8]) {
        // Returns if there are no more splits to be made
        if photons.len() <= 1 {
            return;
        }

        // Finds the bounding box surrounding the photons
        let bounding_box = AABB::from_points(photons.iter().map(|p| p.position).collect());

        // Selects the dimension in which the bounding box is the largest for splitting
        let axis = (0..3)
            .map(|i| (i, bounding_box.get_max(i) - bounding_box.get_min(i)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;

        // Finds the median of the points in that dimension
        let median_index = photons.len() / 2;
        photons.select_nth_unstable_by(median_index, |a, b| {
            a.position[axis].partial_cmp(&b.position[axis]).unwrap()
        });
        splits[median_index] = axis;

        // Gets the slices for the subtrees
        let (l_tree, r_tree) = photons.split_at_mut(median_index);
        let (l_splits, r_splits) = splits.split_at_mut(median_index);

        // Recursively builds both subtrees in parallel
        rayon::join(
            || Self::build(l_tree, l_splits),
            || Self::build(&mut r_tree[1..], &mut r_splits[1..]),
        );
    }

    /// Returns the `n` closest photons that lie within the given `radius` of point `x`.
    pub fn find(&self, x: Vertex, n: usize, max_radius: &mut f64) -> Vec<&Photon> {
        // Creates heap to add photons to along with the initial threshold distance
        let mut closest_photons = BinaryHeap::with_capacity(n);
        let mut max_sqr_dist = *max_radius * *max_radius;

        // Starts recursion to locate the photons
        Self::locate_photons(
            &self.tree,
            &self.splits,
            &x,
            &mut max_sqr_dist,
            n,
            &mut closest_photons,
        );

        *max_radius = max_sqr_dist.sqrt();

        // Converts the heap into a vector of photon references
        closest_photons
            .into_sorted_vec()
            .into_iter()
            .map(|pd| pd.photon)
            .collect()
    }

    /// Locates up to the `n` closest photons in the given tree that are within
    /// the specified maximum squared distance from the point `x`.
    ///
    /// # Arguments
    ///
    /// * `tree`: Tree of photons to be searched.
    /// * `splits`: The axis along which each element of the tree was split.
    /// * `x`: The point to find photons close to.
    /// * `max_sqr_dist`: The maximum squared distance a photon can be from `x` to be counted.
    /// * `n`: The maximum number of photons to return.
    /// * `heap`: A reference to a max-heap containing the nearest photons and their distances from `x`.
    fn locate_photons<'a>(
        tree: &'a [Photon],
        splits: &[u8],
        x: &Vertex,
        max_sqr_dist: &mut f64,
        n: usize,
        heap: &mut BinaryHeap<PhotonDistance<'a>>,
    ) {
        // Returns if an empty tree is given
        if tree.len() == 0 {
            return;
        }

        // Gets the current photon
        let root_i = tree.len() / 2;
        let axis = splits[root_i];
        let current_photon = &tree[root_i];

        // Searches child nodes if the current node is not a leaf
        if tree.len() > 2 {
            // Gets signed distance between the point x and the splitting plane
            let delta = x[axis] - current_photon.position[axis];

            // Orders the left and right subtrees based on the position of point `x`
            let (subtrees, splits) = if delta < 0.0 {
                (
                    (&tree[..root_i], &tree[root_i + 1..]),
                    (&splits[..root_i], &splits[root_i + 1..]),
                )
            } else {
                (
                    (&tree[root_i + 1..], &tree[..root_i]),
                    (&splits[root_i + 1..], &splits[..root_i]),
                )
            };

            // Search near subtree
            Self::locate_photons(subtrees.0, splits.0, x, max_sqr_dist, n, heap);

            // Search far subtree
            if delta * delta < *max_sqr_dist {
                Self::locate_photons(subtrees.1, splits.1, x, max_sqr_dist, n, heap);
            }
        } else if tree.len() == 2 {
            // Handles the case where there is only one subtree
            Self::locate_photons(&tree[..root_i], &splits[..root_i], x, max_sqr_dist, n, heap);
        }

        // Handles insertion of the current node
        let sqr_delta = (*x - current_photon.position).len_sqr();
        if sqr_delta < *max_sqr_dist {
            // Inserts the current photon (with its distance) to the heap
            let pd = PhotonDistance {
                photon: current_photon,
                distance: sqr_delta,
            };
            heap.push(pd);

            // Removes the furthest element and updates threshold distance if the heap is too full
            if heap.len() > n {
                heap.pop();
                *max_sqr_dist = heap.peek().unwrap().distance;
            }
        }
    }

    pub fn estimate_radiance<F>(
        &self,
        hit: &Hit,
        photon_colour_fn: F,
        samples: usize,
        max_radius: f64,
        filter_type: FilterType,
    ) -> Colour
    where
        F: Fn(&Photon) -> Colour,
    {
        // Finds the nearest photons in the photon map
        let mut r = max_radius;
        let photons = self.find(hit.position, samples, &mut r);

        // Precomputes denominator for cone filter calculation
        let w_denom_recip = if filter_type == FilterType::Cone {
            1.0 / (CONE_FILTER_K * r)
        } else {
            0.0
        };

        // Loop through each photon to accumulate the radiance estimate
        let mut estimate = Colour::black();
        for photon in photons {
            // Calculates colour of an individual photon
            let mut photon_colour = photon_colour_fn(photon);

            // Applies the cone filter to the photon colour
            if filter_type == FilterType::Cone {
                photon_colour *= 1.0 - (photon.position - hit.position).length() * w_denom_recip
            }

            // Adds the photon colour to the estimate
            estimate += photon_colour
        }

        // Calculate denominator of the final radiance estimate based on the filter
        let denominator = match filter_type {
            FilterType::Disk => PI * r * r,
            FilterType::Sphere => (4.0 / 3.0) * PI * r * r * r,
            FilterType::Cone => (1.0 - (2.0 / (3.0 * CONE_FILTER_K))) * PI * r * r,
        };

        estimate / denominator
    }
}

#[derive(PartialEq)]
/// Struct used to store a photon and a distance for use in a binary heap.
struct PhotonDistance<'a> {
    distance: f64,
    photon: &'a Photon,
}

/// Implements equating photon distances.
impl<'a> Eq for PhotonDistance<'a> {}

/// Implements partial ordering of photons by their distances.
impl<'a> PartialOrd<Self> for PhotonDistance<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implements ordering photons by their distances.
impl<'a> Ord for PhotonDistance<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}
