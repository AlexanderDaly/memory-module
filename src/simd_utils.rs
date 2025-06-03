use core::simd::{Simd, SimdFloat};

/// SIMD-accelerated dot product for `f32` slices.
///
/// Returns 0.0 if the slices are of different lengths.
pub(crate) fn dot(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    const LANES: usize = 8;
    let chunks = a.len() / LANES;
    let remainder = a.len() % LANES;

    let mut sum = Simd::<f32, LANES>::splat(0.0);
    for i in 0..chunks {
        let start = i * LANES;
        let va = Simd::from_slice(&a[start..start + LANES]);
        let vb = Simd::from_slice(&b[start..start + LANES]);
        sum += va * vb;
    }

    let mut result = sum.reduce_sum();
    for i in (a.len() - remainder)..a.len() {
        result += a[i] * b[i];
    }
    result
}

/// Calculates the Euclidean norm of a vector using SIMD.
pub(crate) fn norm(a: &[f32]) -> f32 {
    const LANES: usize = 8;
    let chunks = a.len() / LANES;
    let remainder = a.len() % LANES;

    let mut sum = Simd::<f32, LANES>::splat(0.0);
    for i in 0..chunks {
        let start = i * LANES;
        let va = Simd::from_slice(&a[start..start + LANES]);
        sum += va * va;
    }

    let mut result = sum.reduce_sum();
    for i in (a.len() - remainder)..a.len() {
        result += a[i] * a[i];
    }

    result.sqrt()
}

/// Computes cosine similarity between two vectors using SIMD.
pub(crate) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let dot_product = dot(a, b);
    let norm_a = norm(a);
    let norm_b = norm(b);

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
