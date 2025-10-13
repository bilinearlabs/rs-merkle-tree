// Copyright 2025 Bilinear Labs - MIT License

//! Store module contains the trait definition and some common elements used by the stores.

use crate::{errors::MerkleError, node::Node};

/// Trait that defines a generic API to store and retrieve nodes of a Merkle tree.
/// TODO: Maybe add depth as generic for safety
pub trait Store {
    // TODO: Explore if making this zero copy is worth it.
    fn get(&self, levels: &[u32], indices: &[u64]) -> Result<Vec<Option<Node>>, MerkleError>;
    fn put(&mut self, items: &[(u32, u64, Node)]) -> Result<(), MerkleError>;
    fn get_num_leaves(&self) -> u64;
}
