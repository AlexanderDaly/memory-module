//! SIMD-accelerated vector operations.
//!
//! This module provides optimized implementations for common vector
//! calculations using architecture intrinsics when the `simd` feature
//! is enabled. If SIMD is not available or the feature is disabled,
//! scalar fallbacks are used instead.

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use std::arch::x86_64::*;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut sum = _mm_setzero_ps();
    let mut sum_a = _mm_setzero_ps();
    let mut sum_b = _mm_setzero_ps();
    let chunks = a.len() / 4;
    let ptr_a = a.as_ptr();
    let ptr_b = b.as_ptr();
    for i in 0..chunks {
        let va = _mm_loadu_ps(ptr_a.add(i * 4));
        let vb = _mm_loadu_ps(ptr_b.add(i * 4));
        sum = _mm_add_ps(sum, _mm_mul_ps(va, vb));
        sum_a = _mm_add_ps(sum_a, _mm_mul_ps(va, va));
        sum_b = _mm_add_ps(sum_b, _mm_mul_ps(vb, vb));
    }

    let mut dot_arr = [0f32; 4];
    let mut norm_a_arr = [0f32; 4];
    let mut norm_b_arr = [0f32; 4];
    _mm_storeu_ps(dot_arr.as_mut_ptr(), sum);
    _mm_storeu_ps(norm_a_arr.as_mut_ptr(), sum_a);
    _mm_storeu_ps(norm_b_arr.as_mut_ptr(), sum_b);

    let mut dot = dot_arr.iter().sum::<f32>();
    let mut norm_a = norm_a_arr.iter().sum::<f32>();
    let mut norm_b = norm_b_arr.iter().sum::<f32>();

    for i in (chunks * 4)..a.len() {
        let x = *a.get_unchecked(i);
        let y = *b.get_unchecked(i);
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub(crate) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: function uses SSE2 instructions which are available on x86_64
    unsafe { cosine_similarity_simd(a, b) }
}

#[cfg(any(not(feature = "simd"), not(target_arch = "x86_64")))]
pub(crate) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b).map(|(&x, &y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|&x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|&x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
