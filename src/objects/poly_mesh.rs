use crate::core::{Hit, Ray};
use crate::drawing::TexCoords;
use crate::objects::{KDTree, ObjectTrait, Triangle};
use crate::utils::yaml::{parse_bool, parse_string, parse_transforms, FromYaml, YamlPropertyError};
use crate::utils::{read_file, Transform, Vector, Vertex, AABB};
use std::str::FromStr;
use std::time::Instant;
use std::usize;
use yaml_rust::Yaml;

#[derive(Debug)]
/// Object that reads and represents a Wavefront OBJ triangle mesh.
/// Acts as a wrapper for a KDTree that contains the mesh's triangles.
pub struct PolyMesh {
    tree: KDTree,
}

impl PolyMesh {
    /// Error messages for parsing.
    const VERTEX_ERR: &'static str = "Error occurred while parsing: invalid vertex format.";
    const TEX_COORDS_ERR: &'static str =
        "Error occurred while parsing: invalid texture coordinates format.";
    const VECTOR_ERR: &'static str = "Error occurred while parsing: invalid normal vector format.";
    const FACE_ERR: &'static str = "Error occurred while parsing: invalid face format.";

    /// Constructs a new PolyMesh from a given OBJ file.
    /// If no normals are specified in the file, they will be automatically generated using face
    /// normals.
    ///
    /// # Arguments
    ///
    /// * `filename`: Filename of the mesh to load.
    /// * `material`: Material of the mesh.
    pub fn new(filename: &str, material: &str) -> PolyMesh {
        Self::construct_mesh(filename, true, true, material)
    }

    /// Constructs a new PolyMesh from a given OBJ file, generating normals from each face
    /// with optional smoothing. If normals are specified in the file, they will be ignored.
    /// If smoothing is enabled, vertex normals will be calculated as the average of their
    /// surrounding face normals.
    ///
    /// # Arguments
    ///
    /// * `filename`: Filename of the mesh to load.
    /// * `smooth`: Whether to apply smoothing to the generated normals.
    /// * `material`: Material of the mesh.
    pub fn with_generated_normals(filename: &str, smooth: bool, material: &str) -> PolyMesh {
        Self::construct_mesh(filename, false, smooth, material)
    }

    /// Constructs a mesh using the provided flags.
    /// This method is not for direct use, but is used by other PolyMesh constructors to provide
    /// their expected outcomes.
    ///
    /// # Arguments
    ///
    /// * `filename`: Filename of the mesh to load.
    /// * `read_normals`: Whether to read normals from the file.
    /// * `smooth`: Whether to apply smoothing to generated normals.
    /// * `material`: Material of the mesh.
    fn construct_mesh(
        filename: &str,
        read_normals: bool,
        smooth: bool,
        material: &str,
    ) -> PolyMesh {
        // Starts loading timer and reads in the contents of the file
        let start_time = Instant::now();
        let contents = read_file(filename);

        // Vectors to store parsed information about the mesh
        let mut vertices = Vec::new();
        let mut textures = Vec::new();
        let mut normals = Vec::new();

        // Vectors to store parsed indices for each face in the mesh
        let mut face_vs = Vec::new();
        let mut face_vts = Vec::new();
        let mut face_vns = Vec::new();

        // Reads and parses each line in turn
        for line in contents.lines().map(|x| x.trim()) {
            // Splits the line by whitespace into parts
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Attempts to match the parts to a possible case
            match parts.as_slice() {
                // Reads in vertices of all forms
                ["v", x, y, z] => vertices.push(Self::parse_vertex(x, y, z, &"1.0")),
                ["v", x, y, z, w] => vertices.push(Self::parse_vertex(x, y, z, w)),

                // Reads in vertex texture coordinates of all forms
                ["vt", u] => textures.push(Self::parse_tex_coords(u, &"0.0", &"0.0")),
                ["vt", u, v] => textures.push(Self::parse_tex_coords(u, v, &"0.0")),
                ["vt", u, v, w] => textures.push(Self::parse_tex_coords(u, v, w)),

                // Reads in vertex normals
                ["vn", x, y, z] => {
                    if read_normals {
                        normals.push(Self::parse_vector(x, y, z))
                    }
                }

                // Reads in details about a face in the model
                ["f", face_parts @ ..] => {
                    let (vs, vts, vns) = Self::split_face_parts(face_parts);
                    face_vs.extend(vs);
                    face_vts.extend(vts);
                    face_vns.extend(vns);
                }

                // Skips lines which don't match any of the specified formats
                _ => {}
            }
        }

        // Handle case where face normals need to be calculated, including smoothing
        if normals.len() == 0 || face_vns.len() == 0 {
            (normals, face_vns) = Self::calculate_face_normals(&vertices, &face_vs, smooth);
        }

        // Constructs all Triangle objects for the mesh
        let mut triangles = Vec::with_capacity(face_vs.len());
        for i in 0..face_vs.len() {
            // Gets triangle properties from indices
            let vertices = face_vs[i].map(|j| vertices[j]);
            let normals = face_vns[i].map(|j| normals[j]);

            // Gets texture coordinates as an optional property
            let tex_coords = if face_vts.len() > 0 {
                face_vts[i]
                    .map(|j| textures.get(j).cloned())
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .and_then(|v| v.try_into().ok())
            } else {
                None
            };

            // Constructs triangle and pushes its object to the vector
            let triangle = Triangle::new(vertices, normals, tex_coords, material);
            triangles.push(triangle.into());
        }

        let tree = KDTree::new(triangles);

        println!(
            "Loaded mesh '{}' with {} triangles in {:?}.",
            filename,
            face_vs.len(),
            start_time.elapsed()
        );

        PolyMesh { tree }
    }

    /// Parses given x, y, z, and w string slices into a Vertex object.
    fn parse_vertex(x: &&str, y: &&str, z: &&str, w: &&str) -> Vertex {
        Vertex::with_w(
            f64::from_str(x).expect(Self::VERTEX_ERR),
            f64::from_str(y).expect(Self::VERTEX_ERR),
            f64::from_str(z).expect(Self::VERTEX_ERR),
            f64::from_str(w).expect(Self::VERTEX_ERR),
        )
    }

    /// Parses given u, v, and w string slices into texture coordinates.
    fn parse_tex_coords(u: &&str, v: &&str, w: &&str) -> TexCoords {
        TexCoords::new(
            f64::from_str(u).expect(Self::TEX_COORDS_ERR),
            f64::from_str(v).expect(Self::TEX_COORDS_ERR),
            f64::from_str(w).expect(Self::TEX_COORDS_ERR),
        )
    }

    /// Parses given x, y, and z string slices into a normalised Vector object.
    fn parse_vector(x: &&str, y: &&str, z: &&str) -> Vector {
        Vector::new(
            f64::from_str(x).expect(Self::VECTOR_ERR),
            f64::from_str(y).expect(Self::VECTOR_ERR),
            f64::from_str(z).expect(Self::VECTOR_ERR),
        )
        .unit()
    }

    /// Splits and parses each possible component of a face into distinct indices lists.
    fn split_face_parts(
        face_parts: &[&str],
    ) -> (Vec<[usize; 3]>, Vec<[usize; 3]>, Vec<[usize; 3]>) {
        // Initialises vectors to store components of the face
        let mut vs = Vec::new();
        let mut vts = Vec::new();
        let mut vns = Vec::new();

        // Extracts v, vt, and vn values as zero-based indices
        for part in face_parts {
            let components: Vec<&str> = part.split("/").collect();

            match components.as_slice() {
                // Handles the case with just vertices
                [v] => vs.push(usize::from_str(v).expect(Self::VECTOR_ERR) - 1),

                // Handles the case with vertices and texture coordinates
                [v, vt] => {
                    vs.push(usize::from_str(v).expect(Self::VECTOR_ERR) - 1);
                    vts.push(usize::from_str(vt).expect(Self::VECTOR_ERR) - 1);
                }

                // Handles the case with vertices, normals and (optionally) texture coordinates
                [v, vt, vn] => {
                    vs.push(usize::from_str(v).expect(Self::VECTOR_ERR) - 1);
                    vns.push(usize::from_str(vn).expect(Self::VECTOR_ERR) - 1);

                    match usize::from_str(vt) {
                        Ok(vt) => vts.push(vt - 1),
                        Err(..) => continue,
                    };
                }

                // Handles incorrect face formatting
                _ => panic!("{}", Self::FACE_ERR),
            }
        }

        // Sets a minimum size to be validly split into triangles
        let min_size = face_parts.len();

        // Split each component list into triangles if it meets the minimum size, and return
        (
            Self::split_into_triangles(vs, min_size),
            Self::split_into_triangles(vts, min_size),
            Self::split_into_triangles(vns, min_size),
        )
    }

    /// Splits a list of indices given by a face into indices for each triangle in the face.
    fn split_into_triangles(components: Vec<usize>, min_size: usize) -> Vec<[usize; 3]> {
        if components.len() < min_size {
            Vec::with_capacity(0)
        } else {
            (1..(components.len() - 1))
                .map(|i| [components[0], components[i], components[i + 1]])
                .collect()
        }
    }

    /// Calculates the normals for each face in a mesh using the vertex information, with optional smoothing
    fn calculate_face_normals(
        vertices: &Vec<Vertex>,
        face_vs: &Vec<[usize; 3]>,
        smoothing: bool,
    ) -> (Vec<Vector>, Vec<[usize; 3]>) {
        // Vector to store the normals of each face
        let mut face_normals = Vec::with_capacity(face_vs.len());

        // Uses the cross product to calculate the normal of each face using its vertices
        for i in 0..face_vs.len() {
            let points = face_vs[i].map(|j| vertices[j]);
            let edge1 = points[1] - points[0];
            let edge2 = points[2] - points[0];
            face_normals.push(Vector::cross(&edge1, &edge2).unit());
        }

        // Normals and indices vectors to be returned
        let mut normals;
        let face_vns;

        // Constructs normals and indices vectors
        if smoothing {
            // If smoothing, uses an average of surrounding faces' vertices for smoothness
            normals = vec![Vector::zero(); vertices.len()];
            face_vns = face_vs.clone();

            // Calculates sum of face normals surrounding each vertex
            for (i, indices) in face_vs.iter().enumerate() {
                for &j in indices {
                    normals[j] += face_normals[i];
                }
            }

            // Normalises the summed vectors to get an average vector
            normals.iter_mut().for_each(|n| n.normalise());
        } else {
            // If not smoothing, simply uses the face normal for every vertex in a face
            normals = face_normals;
            face_vns = (0..face_vs.len()).map(|i| [i, i, i]).collect();
        }

        (normals, face_vns)
    }
}

impl ObjectTrait for PolyMesh {
    fn intersection(&self, ray: &Ray) -> Vec<Hit<'_>> {
        self.tree.intersection(ray)
    }

    fn apply_transform(&mut self, transform: Transform) {
        self.tree.apply_transform(transform)
    }

    fn get_bounding_box(&self) -> &AABB {
        self.tree.get_bounding_box()
    }
}

/// Implements loading a `PolyMesh` from a YAML file.
impl FromYaml for PolyMesh {
    fn from_yaml(yaml: &Yaml) -> Result<PolyMesh, YamlPropertyError> {
        // Parses properties for the poly mesh
        let filename = parse_string(yaml, "filename")?;
        let material = parse_string(yaml, "material")?;

        // Creates the `PolyMesh` instance
        let mut mesh = match parse_bool(yaml, "smooth") {
            Ok(smooth) => PolyMesh::with_generated_normals(&filename, smooth, &material),
            Err(_) => PolyMesh::new(&filename, &material),
        };

        // Applies any present transforms to the mesh
        let transform = parse_transforms(yaml)?;
        mesh.apply_transform(transform);

        Ok(mesh)
    }
}
