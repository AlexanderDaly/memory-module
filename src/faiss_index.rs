#[cfg(feature = "faiss")]
use faiss::{index::flat::FlatIndex, index::id_map::IdMap, metric::MetricType, IndexImpl};
#[cfg(feature = "faiss")]
use std::collections::HashMap;
#[cfg(feature = "faiss")]
use uuid::Uuid;

#[cfg(feature = "faiss")]
/// Wrapper around a FAISS index for storing memory embeddings.
pub struct FaissIndex {
    index: IdMap<FlatIndex>,
    dim: usize,
    next_id: u64,
    map: HashMap<u64, Uuid>,
}

#[cfg(feature = "faiss")]
impl FaissIndex {
    /// Create a new FAISS index with the given dimensionality.
    pub fn new(dim: usize) -> faiss::error::Result<Self> {
        let quantizer = FlatIndex::new(dim as u32, MetricType::L2)?;
        let index = IdMap::new(quantizer)?;
        Ok(Self { index, dim, next_id: 0, map: HashMap::new() })
    }

    /// Add a vector with the associated memory `Uuid`.
    pub fn add_vector(&mut self, id: Uuid, vector: &[f32]) -> faiss::error::Result<()> {
        assert_eq!(vector.len(), self.dim, "Vector dimension mismatch");
        let faiss_id = self.next_id;
        self.next_id += 1;
        self.map.insert(faiss_id, id);
        self.index.add_with_ids(vector, &[faiss_id])?;
        Ok(())
    }

    /// Search for nearest neighbours of the query vector.
    pub fn search(&self, query: &[f32], k: usize) -> faiss::error::Result<Vec<(f32, Uuid)>> {
        if query.len() != self.dim {
            return Ok(Vec::new());
        }
        let (distances, ids) = self.index.search(query, k)?;
        let results = distances
            .into_iter()
            .zip(ids.into_iter())
            .filter_map(|(d, fid)| self.map.get(&fid).map(|uid| (d, *uid)))
            .collect();
        Ok(results)
    }
}

#[cfg(not(feature = "faiss"))]
/// Dummy index used when the `faiss` feature is disabled.
pub struct FaissIndex;

#[cfg(not(feature = "faiss"))]
impl FaissIndex {
    pub fn new(_dim: usize) -> Result<Self, ()> { Ok(Self) }
    pub fn add_vector(&mut self, _id: uuid::Uuid, _v: &[f32]) -> Result<(), ()> { Ok(()) }
    pub fn search(&self, _q: &[f32], _k: usize) -> Result<Vec<(f32, uuid::Uuid)>, ()> { Ok(Vec::new()) }
}

