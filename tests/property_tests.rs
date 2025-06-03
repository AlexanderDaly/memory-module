use memory_module::simd_utils;
use proptest::prelude::*;

proptest! {
    #[test]
    fn cosine_similarity_is_symmetric(a in proptest::collection::vec(-1.0f32..1.0, 0..8),
                                      b in proptest::collection::vec(-1.0f32..1.0, 0..8)) {
        let ab = simd_utils::cosine_similarity(&a, &b);
        let ba = simd_utils::cosine_similarity(&b, &a);
        prop_assert!((ab - ba).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_self_is_one(v in proptest::collection::vec(-1.0f32..1.0, 1..8)) {
        let cs = simd_utils::cosine_similarity(&v, &v);
        prop_assert!((cs - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_similarity_in_range(a in proptest::collection::vec(-1.0f32..1.0, 0..8),
                                  b in proptest::collection::vec(-1.0f32..1.0, 0..8)) {
        let cs = simd_utils::cosine_similarity(&a, &b);
        prop_assert!(cs >= -1.0 && cs <= 1.0);
    }
}
