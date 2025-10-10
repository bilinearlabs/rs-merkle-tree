// Copyright 2025 Bilinear Labs - MIT License

//! Sled store implementation.

#[cfg(feature = "sled_store")]
use crate::{MerkleError, Node, Store};
#[cfg(feature = "sled_store")]
use sled::{Batch, Db};

#[cfg(feature = "sled_store")]
pub struct SledStore {
    db: Db,

    // Using an in memory counter to avoid reading the db for the number of leaves.
    num_leaves: u64,
}

#[cfg(feature = "sled_store")]
impl SledStore {
    const KEY_NUM_LEAVES: &'static [u8] = b"NUM_LEAVES";

    fn db_error<E: std::fmt::Display>(err: E) -> MerkleError {
        MerkleError::StoreError(err.to_string())
    }

    fn encode_key(level: u32, index: u64) -> [u8; 12] {
        let mut key = [0u8; 12];

        key[..4].copy_from_slice(&level.to_be_bytes());
        key[4..].copy_from_slice(&index.to_be_bytes());

        key
    }

    fn decode_node(bytes: &[u8]) -> Result<Node, MerkleError> {
        // TODO: Options to allow zero copy? Eg using lifetimes on Node?
        let arr: [u8; Node::LEN] = bytes
            .try_into()
            .map_err(|_| MerkleError::StoreError("invalid node length".into()))?;
        Ok(Node::from(arr))
    }
}

#[cfg(feature = "sled_store")]
impl SledStore {
    // TODO: Maybe return result
    pub fn new(file_path: &str, temporary: bool) -> Self {
        // Stuff that can be tunned, unused by now:
        // - mode (small vs fast)
        // - compression
        // - cache capacity
        let db = sled::Config::new()
            .path(file_path)
            .temporary(temporary)
            .open()
            .expect("failed to open sled DB");

        // Load the persisted leaf count (big-endian u64) or default to 0.
        let num_leaves = db
            .get(Self::KEY_NUM_LEAVES)
            .expect("failed to get num leaves")
            .map(|ivec| {
                let bytes: [u8; 8] = ivec.as_ref().try_into().expect("invalid num_leaves length");
                u64::from_be_bytes(bytes)
            })
            .unwrap_or(0);

        // If the tree has no leaves, clear the db just in case.
        if num_leaves == 0 {
            db.clear().expect("failed to clear db");
        }

        Self { db, num_leaves }
    }
}

#[cfg(feature = "sled_store")]
impl Store for SledStore {
    fn get(&self, level: u32, index: u64) -> Result<Option<Node>, MerkleError> {
        let key = Self::encode_key(level, index);
        match self.db.get(&key).map_err(Self::db_error)? {
            None => Ok(None),
            Some(ivec) => Ok(Some(Self::decode_node(&ivec)?)),
        }
    }

    fn put(&mut self, items: &[(u32, u64, Node)]) -> Result<(), MerkleError> {
        let mut batch = Batch::default();

        for (level, index, node) in items.iter() {
            let key = Self::encode_key(*level, *index);
            batch.insert(&key, node.as_ref());
        }

        let counter = items.iter().filter(|(level, _, _)| *level == 0).count();
        batch.insert(
            Self::KEY_NUM_LEAVES,
            &(self.num_leaves + counter as u64).to_be_bytes(),
        );

        self.db.apply_batch(batch).map_err(Self::db_error)?;
        self.num_leaves += counter as u64;

        Ok(())
    }

    fn get_num_leaves(&self) -> u64 {
        self.num_leaves
    }
}
