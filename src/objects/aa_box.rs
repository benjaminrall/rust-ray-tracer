use crate::core::{Hit, Ray};
use crate::objects::{ObjectTrait, Quad};
use crate::utils::yaml::{
    parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, Vector, Vertex, AABB};
use yaml_rust::Yaml;

#[derive(Debug)]
/// Axis-aligned box primitive.
pub struct AABox {
    faces: [Quad; 6], // Faces that make up the box

    bounding_box: AABB,   // Bounding box of the box
    transform: Transform, // Transform of the box
}

impl AABox {
    /// Creates a new axis-aligned box with a given centre point and size in each direction.
    pub fn new(centre: Vertex, size: Vector, material: &str) -> AABox {
        let positive_corner = centre + size;
        let negative_corner = centre - size;
        let box_span = 2.0 * size;

        // Front facing quad
        let front = Quad::new(
            positive_corner,
            Vector::new(-box_span.x, 0.0, 0.0),
            Vector::new(0.0, -box_span.y, 0.0),
            material,
        );

        // Back facing quad
        let back = Quad::new(
            negative_corner,
            Vector::new(0.0, box_span.y, 0.0),
            Vector::new(box_span.x, 0.0, 0.0),
            material,
        );

        // Upwards facing quad
        let up = Quad::new(
            positive_corner,
            Vector::new(0.0, 0.0, -box_span.z),
            Vector::new(-box_span.x, 0.0, 0.0),
            material,
        );

        // Downwards facing quad
        let down = Quad::new(
            negative_corner,
            Vector::new(box_span.x, 0.0, 0.0),
            Vector::new(0.0, 0.0, box_span.z),
            material,
        );

        // Right facing quad
        let right = Quad::new(
            positive_corner,
            Vector::new(0.0, -box_span.y, 0.0),
            Vector::new(0.0, 0.0, -box_span.z),
            material,
        );

        // Left facing quad
        let left = Quad::new(
            negative_corner,
            Vector::new(0.0, 0.0, box_span.z),
            Vector::new(0.0, box_span.y, 0.0),
            material,
        );

        AABox {
            faces: [front, back, up, down, right, left],
            bounding_box: AABB::from_points(vec![negative_corner, positive_corner]),
            transform: Transform::identity(),
        }
    }
}

impl ObjectTrait for AABox {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Transforms ray into the box's object space
        let ray = ray.to_object_space(&self.transform);

        // Iterates over each face of the box and stores all hits
        let mut hits = Vec::with_capacity(2);
        for face in self.faces.iter() {
            hits.extend(face.intersection(&ray))
        }

        // Transforms the hits back into world space
        hits.iter_mut().for_each(|h| h.transform(&self.transform));

        // Sorts the hits
        hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
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

/// Implements loading an `AABox` from a YAML file.
impl FromYaml for AABox {
    fn from_yaml(yaml: &Yaml) -> Result<AABox, YamlPropertyError> {
        // Parses properties for the box
        let centre = parse_struct(yaml, "centre")?;
        let size = parse_struct(yaml, "size")?;
        let material = parse_string(yaml, "material")?;

        // Creates the box instance
        let mut aa_box = AABox::new(centre, size, &material);

        // Applies any present transforms to the box
        let transform = parse_transforms(yaml)?;
        aa_box.apply_transform(transform);

        Ok(aa_box)
    }
}
