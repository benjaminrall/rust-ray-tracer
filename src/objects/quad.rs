use crate::core::{Hit, Ray};
use crate::objects::ObjectTrait;
use crate::utils::yaml::{
    parse_string, parse_struct, parse_transforms, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, Vector, Vertex, AABB};
use crate::EPSILON;
use std::ops::Neg;
use yaml_rust::Yaml;
use crate::drawing::TexCoords;

#[derive(Debug)]
/// Four vertex quadrilateral primitive.
pub struct Quad {
    point: Vertex, // Starting point of the quadrilateral
    edge1: Vector, // First side of the quadrilateral
    edge2: Vector, // Second side of the quadrilateral

    material: String,     // Material of the quad
    bounding_box: AABB,   // Bounding box of the quad
    transform: Transform, // Transform of the quad

    normal: Vector, // Normal of the quad
}

impl Quad {
    pub fn new(point: Vertex, edge1: Vector, edge2: Vector, material: &str) -> Quad {
        // Calculates the bounding box of the quad's vertices
        let bounding_box = AABB::from_points(vec![
            point,
            point + edge1,
            point + edge2,
            point + edge1 + edge2,
        ]);

        // Calculates the normal of the quad
        let normal = Vector::cross(&edge1, &edge2).unit();

        Quad {
            point,
            edge1,
            edge2,
            material: String::from(material),
            bounding_box,
            transform: Transform::identity(),
            normal,
        }
    }
}

impl ObjectTrait for Quad {
    /// Returns all points of intersection, sorted by `t`, if the given ray intersects the object.
    ///
    /// Code uses an adapted form of the Möller–Trumbore ray-triangle intersection used in `Triangle`,
    /// with adjusted conditions for what is considered inside the primitive after calculating barycentric coordinates.
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Transforms ray into the quad's object space
        let ray = ray.to_object_space(&self.transform);

        // Calculates determinant
        let p_vec = Vector::cross(&ray.direction, &self.edge2);
        let det = Vector::dot(&self.edge1, &p_vec);

        // If determinant is near zero, ray lies in plane of quad, so don't count a hit
        if det.abs() < EPSILON {
            return vec![];
        }
        let det_recip = 1.0 / det;

        // Calculates distance from the first vertex to the ray's origin position
        let t_vec = ray.position - self.point;

        // Calculates `u` parameter and tests its bounds
        let u = Vector::dot(&t_vec, &p_vec) * det_recip;
        if u < 0.0 || u > 1.0 {
            return Vec::new();
        }

        // Calculates `v` parameter and tests its bounds
        let q_vec = Vector::cross(&t_vec, &self.edge1);
        let v = Vector::dot(&ray.direction, &q_vec) * det_recip;
        if v < 0.0 || v > 1.0 {
            return Vec::new();
        }

        // Ray intersects quad, calculate t and hit position
        let t = Vector::dot(&self.edge2, &q_vec) * det_recip;
        let pos = ray.position + t * ray.direction;

        // Transform the hit back into world space and return it
        let entering = Vector::dot(&self.normal, &ray.direction) < 0.0;

        let normal = if Vector::dot(&self.normal, &ray.direction) > 0.0 {
            -self.normal
        } else {
            self.normal
        };

        let mut hit = Hit::new(t, entering, pos, normal, &self.material, Some(TexCoords::new(u, v, 0.0)));
        hit.transform(&self.transform);

        vec![hit]
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

/// Implements loading a `Quad` from a YAML file.
impl FromYaml for Quad {
    fn from_yaml(yaml: &Yaml) -> Result<Quad, YamlPropertyError> {
        // Parses properties for the quad
        let point = parse_struct(yaml, "point")?;
        let edge1 = parse_struct(yaml, "edge1")?;
        let edge2 = parse_struct(yaml, "edge2")?;
        let material = parse_string(yaml, "material")?;

        // Creates the quad instance
        let mut quad = Quad::new(point, edge1, edge2, &material);

        // Applies any present transforms to the quad
        let transform = parse_transforms(yaml)?;
        quad.apply_transform(transform);

        Ok(quad)
    }
}
