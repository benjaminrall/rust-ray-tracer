use crate::core::{Hit, Ray};
use crate::objects::ObjectTrait;
use crate::utils::yaml::{
    parse_float_array, parse_string, parse_transforms, parse_vec, FromYaml, YamlPropertyError,
};
use crate::utils::{Transform, Vector, Vertex, AABB};
use crate::EPSILON;
use std::ops::Neg;
use yaml_rust::Yaml;

#[derive(Debug)]
/// A surface represented by a general quadratic equation in x, y, and z.
pub struct Quadratic {
    // Coefficients of the quadratic equation
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
    i: f64,
    j: f64,

    material: String, // Material of the surface
}

impl Quadratic {
    /// Creates a new Quadratic surface, which is represented by the following equation:
    ///
    /// ax^2 + 2bxy + 2cxz + 2dx + ey^2 + 2fyz + 2gy + hz^2 + 2iz + j = 0
    ///
    /// # Arguments
    ///
    /// * `a`: Coefficient for x^2.
    /// * `b`: Coefficient for xy.
    /// * `c`: Coefficient for xz.
    /// * `d`: Coefficient for x.
    /// * `e`: Coefficient for y^2.
    /// * `f`: Coefficient for yz.
    /// * `g`: Coefficient for y.
    /// * `h`: Coefficient for z^2.
    /// * `i`: Coefficient for z.
    /// * `j`: Constant term.
    /// * `material`: Material of the surface.
    pub fn new(
        a: f64,
        b: f64,
        c: f64,
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: f64,
        material: &str,
    ) -> Quadratic {
        Quadratic {
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
            i,
            j,
            material: String::from(material),
        }
    }

    pub fn from_values_list(values: Vec<f64>, material: &str) -> Quadratic {
        Quadratic {
            a: values[0],
            b: values[1],
            c: values[2],
            d: values[3],
            e: values[4],
            f: values[5],
            g: values[6],
            h: values[7],
            i: values[8],
            j: values[9],
            material: String::from(material),
        }
    }

    /// Gets the normal of the surface at a given point.
    fn get_normal(&self, point: Vertex) -> Vector {
        Vector::new(
            self.a * point.x + self.b * point.y + self.c * point.z + self.d,
            self.b * point.x + self.e * point.y + self.f * point.z + self.g,
            self.c * point.x + self.f * point.y + self.h * point.z + self.i,
        )
        .unit()
    }
}

impl ObjectTrait for Quadratic {
    fn intersection(&self, ray: &Ray) -> Vec<Hit> {
        // Extract ray position and direction components for readability
        let (px, py, pz) = (ray.position.x, ray.position.y, ray.position.z);
        let (dx, dy, dz) = (ray.direction.x, ray.direction.y, ray.direction.z);

        // Coefficients for the quadratic equation (aq * t^2 + bq * t + cq = 0)
        let aq = self.a * dx * dx
            + 2.0 * self.b * dx * dy
            + 2.0 * self.c * dx * dz
            + self.e * dy * dy
            + 2.0 * self.f * dy * dz
            + self.h * dz * dz;
        let bq = 2.0
            * (self.a * px * dx
                + self.b * (px * dy + dx * py)
                + self.c * (px * dz + dx * pz)
                + self.d * dx
                + self.e * py * dy
                + self.f * (py * dz + dy * pz)
                + self.g * dy
                + self.h * pz * dz
                + self.i * dz);
        let cq = self.a * px * px
            + 2.0 * self.b * px * py
            + 2.0 * self.c * px * pz
            + 2.0 * self.d * px
            + self.e * py * py
            + 2.0 * self.f * py * pz
            + 2.0 * self.g * py
            + self.h * pz * pz
            + 2.0 * self.i * pz
            + self.j;

        // Handles the case when the ray is tangent to the surface
        if aq == 0.0 {
            // Catches no solution case
            if bq == 0.0 {
                return Vec::new();
            }

            // Solves for the hit's t value
            let t = -cq / bq;

            // Calculates the position and normal of the hit
            let pos = ray.position + t * ray.direction;
            let mut normal = self.get_normal(pos);
            if Vector::dot(&normal, &ray.direction) > 0.0 {
                normal = normal.neg();
            }

            // Returns the hit
            let hit = Hit::new(t, true, pos, normal, &self.material, None);
            return vec![hit];
        }

        // Calculates the discriminant to determine intersections
        let discriminant = bq * bq - 4.0 * aq * cq;

        // No intersections if the discriminant is negative
        if discriminant < EPSILON {
            return Vec::new();
        }

        // Calculates roots using the quadratic formula
        let denominator_recip = 0.5 / aq;
        let discriminant_sqrt = discriminant.sqrt();
        let t0 = (-bq - discriminant_sqrt) * denominator_recip;
        let t1 = (-bq + discriminant_sqrt) * denominator_recip;

        // Calculates positions of the intersections
        let pos0 = ray.position + t0 * ray.direction;
        let pos1 = ray.position + t1 * ray.direction;

        // Calculates normals at the points of intersection
        let mut normal0 = self.get_normal(pos0);
        let mut normal1 = self.get_normal(pos1);

        // Negates normals if facing away from the ray
        if Vector::dot(&normal0, &ray.direction) > 0.0 {
            normal0 = normal0.neg();
        }
        if Vector::dot(&normal1, &ray.direction) > 0.0 {
            normal1 = normal1.neg();
        }

        // Creates hit objects
        let hit0 = Hit::new(t0, true, pos0, normal0, &self.material, None);
        let hit1 = Hit::new(t1, false, pos1, normal1, &self.material, None);
        vec![hit0, hit1]
    }

    fn apply_transform(&mut self, transform: Transform) {
        // Creates a matrix from the surface's current parameters
        let q = Transform::new(
            self.a, self.b, self.c, self.d, self.b, self.e, self.f, self.g, self.c, self.f, self.h,
            self.i, self.d, self.g, self.i, self.j,
        );

        // Gets inverse transform matrix
        let ti = transform.get_inverse();

        // Calculates and assigns the new coefficients
        let q_dash = ti.transposed() * (&q * &ti);
        self.a = q_dash[0][0];
        self.b = q_dash[0][1];
        self.c = q_dash[0][2];
        self.d = q_dash[0][3];
        self.e = q_dash[1][1];
        self.f = q_dash[1][2];
        self.g = q_dash[1][3];
        self.h = q_dash[2][2];
        self.i = q_dash[2][3];
        self.j = q_dash[3][3];
    }

    fn get_bounding_box(&self) -> &AABB {
        unimplemented!("Quadratics are an infinite surface without a bounding box.")
    }
}

/// Implements loading a `Quadratic` from a YAML file.
impl FromYaml for Quadratic {
    fn from_yaml(yaml: &Yaml) -> Result<Quadratic, YamlPropertyError> {
        // Parses properties for the quadratic
        let values = parse_vec(yaml, "values")?;
        let float_values = parse_float_array(&values)?;
        let material = parse_string(yaml, "material")?;

        // Ensures the values list is the correct length
        if float_values.len() != 10 {
            return Err(YamlPropertyError::invalid("values"));
        }

        // Creates the quadratic instance
        let mut quadratic = Quadratic::from_values_list(float_values, &material);

        // Applies any present transforms to the quadratic
        let transform = parse_transforms(yaml)?;
        quadratic.apply_transform(transform);

        Ok(quadratic)
    }
}
