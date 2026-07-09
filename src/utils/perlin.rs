// Functions related to generating Perlin noise.
//
// Based on descriptions from:
// An image synthesizer (Ken Perlin, 1985) https://dl.acm.org/doi/10.1145/325165.325247
// Improving noise (Ken Perlin, 2002) https://dl.acm.org/doi/10.1145/566654.566636
// https://mrl.cs.nyu.edu/~perlin/noise/

use crate::utils::Vertex;
use std::ops::MulAssign;

/// Permutations for the hash function, used by Ken Perlin's original implementation of Perlin noise
const P: [usize; 512] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180, 151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194,
    233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234,
    75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174,
    20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83,
    111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25,
    63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188,
    159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147,
    118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
    213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253,
    19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193,
    238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

/// Gradient vectors set to be chosen from at each point on the lattice
const GRADIENTS: [(f64, f64, f64); 16] = [
    (1.0, 1.0, 0.0),
    (-1.0, 1.0, 0.0),
    (1.0, -1.0, 0.0),
    (-1.0, -1.0, 0.0),
    (1.0, 0.0, 1.0),
    (-1.0, 0.0, 1.0),
    (1.0, 0.0, -1.0),
    (-1.0, 0.0, -1.0),
    (0.0, 1.0, 1.0),
    (0.0, -1.0, 1.0),
    (0.0, 1.0, -1.0),
    (0.0, -1.0, -1.0),
    (1.0, 1.0, 0.0),
    (-1.0, 1.0, 0.0),
    (0.0, -1.0, 1.0),
    (0.0, -1.0, -1.0),
];

/// Deterministically hashes given coordinates on the integer cubic lattice,
/// returning a pseudorandom index for the gradients array.
pub fn hash(i: usize, j: usize, k: usize) -> usize {
    P[P[P[i] + j] + k] & 15
}

/// Applies the 'smootherstep' function to a given interpolation value t in [0, 1].
pub fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Performs linear interpolation between two values
pub fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

/// Performs tri-linear interpolation on a set of values
pub fn trilinear_interpolate(values: [f64; 8], u: f64, v: f64, w: f64) -> f64 {
    let x00 = lerp(u, values[0], values[4]);
    let x01 = lerp(u, values[1], values[5]);
    let x10 = lerp(u, values[2], values[6]);
    let x11 = lerp(u, values[3], values[7]);

    let xy0 = lerp(v, x00, x10);
    let xy1 = lerp(v, x01, x11);

    lerp(w, xy0, xy1)
}

/// Generates perlin noise for a 3D point
pub fn noise(point: Vertex) -> f64 {
    // Calculates the floored coordinates of the point
    let fx = point.x.floor();
    let fy = point.y.floor();
    let fz = point.z.floor();

    // Gets the integer grid cell coordinates of the point
    let i = (fx as i32 & 255) as usize;
    let j = (fy as i32 & 255) as usize;
    let k = (fz as i32 & 255) as usize;

    // Calculates the fractional parts of the point
    let x = point.x - fx;
    let y = point.y - fy;
    let z = point.z - fz;

    // Calculates the eased fractional parts of the point for interpolation
    let u = fade(x);
    let v = fade(y);
    let w = fade(z);

    // Calculates the values assigned to each corner of the point's cube
    let mut values = [0.0; 8];
    for p in 0..2 {
        for q in 0..2 {
            for r in 0..2 {
                // Gets the gradient for the current corner
                let g = GRADIENTS[hash(i + p, j + q, k + r)];

                // Calculates offset vector values
                let dx = x - p as f64;
                let dy = y - q as f64;
                let dz = z - r as f64;

                // Calculates dot product value of the corner
                values[p * 4 + q * 2 + r] = g.0 * dx + g.1 * dy + g.2 * dz;
            }
        }
    }

    // Calculates the result of tri-linear interpolation on these values
    trilinear_interpolate(values, u, v, w)
}

/// Generates turbulence for a 3D point by layering different frequencies of absolute noise
pub fn turbulence(point: Vertex, octaves: i32, persistence: f64) -> f64 {
    layered_noise(point, octaves, persistence).abs()
}

/// Generates layered noise for a 3D point
pub fn layered_noise(mut point: Vertex, octaves: i32, persistence: f64) -> f64 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    // Applies each octave of noise
    for _ in 0..octaves {
        total += amplitude * noise(point);
        max_value += amplitude;
        amplitude *= persistence;
        point *= 2.0;
    }

    total / max_value
}
