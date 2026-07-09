//! Primitive objects and aggregate volumes that can be placed in a scene.

mod aa_box;
mod constant_medium;
mod csg;
mod group;
mod instance;
mod instances;
mod kd_tree;
mod object;
mod plane;
mod poly_mesh;
mod quad;
mod quadratic;
mod sphere;
mod triangle;

pub use aa_box::AABox;
pub use constant_medium::ConstantMedium;
pub use csg::{CSGOp, CSG};
pub use group::Group;
pub use instance::Instance;
pub use kd_tree::KDTree;
pub use object::{Object, ObjectTrait};
pub use plane::Plane;
pub use poly_mesh::PolyMesh;
pub use quad::Quad;
pub use quadratic::Quadratic;
pub use sphere::Sphere;
pub use triangle::Triangle;
