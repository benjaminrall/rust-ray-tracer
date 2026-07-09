use crate::core::{Hit, Ray};
use crate::drawing::TexCoords;
use crate::objects::ObjectTrait;
use crate::utils::{Transform, Vector, Vertex, AABB};
use crate::EPSILON;

#[derive(Clone, Debug)]
/// Triangle primitive for handling intersection with meshes.
pub struct Triangle {
    vertices: [Vertex; 3],              // Vertices of the triangle
    normals: [Vector; 3],               // Normal vector at each vertex
    tex_coords: Option<[TexCoords; 3]>, // Texture coordinates at each vertex

    material: String,   // Material of the triangle
    bounding_box: AABB, // Bounding box of the triangle

    // Precomputed edge vectors for faster intersection computation
    edge1: Vector,
    edge2: Vector,
}

impl Triangle {
    /// Creates a new Triangle object from a given list of vertices, normals, and optional texture coords.
    pub fn new(
        vertices: [Vertex; 3],
        normals: [Vector; 3],
        tex_coords: Option<[TexCoords; 3]>,
        material: &str,
    ) -> Triangle {
        // Calculates the bounding box of the triangle's vertices
        let bounding_box = AABB::from_points(vertices.to_vec());

        // Calculates the triangle's two edges to be used for intersection calculations
        let edge1 = vertices[1] - vertices[0];
        let edge2 = vertices[2] - vertices[0];

        Triangle {
            vertices,
            normals,
            tex_coords,
            material: String::from(material),
            bounding_box,
            edge1,
            edge2,
        }
    }
}

impl ObjectTrait for Triangle {
    /// Returns all points of intersection, sorted by `t`, if the given ray intersects the object.
    ///
    /// Code adapted from the Möller–Trumbore ray-triangle intersection algorithm paper, which can
    /// be found here:
    /// https://cadxfem.org/inf/Fast%20MinimumStorage%20RayTriangle%20Intersection.pdf
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Calculates determinant
        let p_vec = Vector::cross(&ray.direction, &self.edge2);
        let det = Vector::dot(&self.edge1, &p_vec);

        // If determinant is near zero, ray lies in plane of triangle, so don't count a hit
        if det.abs() < EPSILON {
            return vec![];
        }
        let det_recip = 1.0 / det;

        // Calculates distance from the first vertex to the ray's origin position
        let t_vec = ray.position - self.vertices[0];

        // Calculates `u` parameter and tests its bounds
        let u = Vector::dot(&t_vec, &p_vec) * det_recip;
        if u < 0.0 || u > 1.0 {
            return Vec::new();
        }

        // Calculates `v` parameter and tests its bounds
        let q_vec = Vector::cross(&t_vec, &self.edge1);
        let v = Vector::dot(&ray.direction, &q_vec) * det_recip;
        if v < 0.0 || u + v > 1.0 {
            return Vec::new();
        }

        // Ray intersects triangle, calculate t and hit position
        let t = Vector::dot(&self.edge2, &q_vec) * det_recip;
        let pos = ray.position + t * ray.direction;

        // Uses barycentric coordinates for normal interpolation
        let normal = (1.0 - u - v) * self.normals[0] + u * self.normals[1] + v * self.normals[2];

        // Uses barycentric coordinates for texture interpolation
        let tex_coords = match self.tex_coords {
            None => None,
            Some(ts) => Some((1.0 - u - v) * ts[0] + u * ts[1] + v * ts[2]),
        };

        // Record a hit
        let (normal, entering) = if Vector::dot(&normal, &ray.direction) < 0.0 {
            (normal, true)
        } else {
            (-normal, false)
        };

        vec![Hit::new(
            t,
            entering,
            pos,
            normal,
            &self.material,
            tex_coords,
        )]
    }

    fn apply_transform(&mut self, transform: Transform) {
        // Applies transform to vertices
        for vertex in self.vertices.iter_mut() {
            transform.apply_vertex(vertex);
        }
        self.edge1 = self.vertices[1] - self.vertices[0];
        self.edge2 = self.vertices[2] - self.vertices[0];

        // Applies transform to normals
        for normal in self.normals.iter_mut() {
            transform.apply_transpose_inverse_vector(normal);
            normal.normalise();
        }

        // Re-calculates bounding box
        self.bounding_box = AABB::from_points(self.vertices.to_vec());
    }

    fn get_bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
