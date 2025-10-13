// Copyright 2025 Bilinear Labs - MIT License

//! Simple in-memory store implementation.

use crate::{MerkleError, Node, Store};
use std::collections::HashMap;

/// Simple in-memory store implementation using a `HashMap`.
#[derive(Default)]
pub struct MemoryStore {
    store: HashMap<(u32, u64), Node>,
    num_leaves: u64,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemoryStore {
    fn get(&self, levels: &[u32], indices: &[u64]) -> Result<Vec<Option<Node>>, MerkleError> {
        if levels.len() != indices.len() {
            return Err(MerkleError::StoreError(
                "levels and indices must have the same length".into(),
            ));
        }

        // Collect the requested nodes in order.
        let mut result = Vec::with_capacity(levels.len());
        for (lvl, idx) in levels.iter().zip(indices.iter()) {
            let node = self.store.get(&(*lvl, *idx)).cloned();
            result.push(node);
        }

        Ok(result)
    }

    fn put(&mut self, items: &[(u32, u64, Node)]) -> Result<(), MerkleError> {
        for (level, index, hash) in items {
            self.store.insert((*level, *index), *hash);
        }
        let counter = items.iter().filter(|(level, _, _)| *level == 0).count();
        self.num_leaves += counter as u64;
        Ok(())
    }
    fn get_num_leaves(&self) -> u64 {
        self.num_leaves
    }
}
