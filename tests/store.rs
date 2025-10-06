// Copyright 2025 Bilinear Labs - MIT License

use rs_merkle_tree::{node::Node, to_node, Store};
use std::fs;

#[cfg(feature = "memory_store")]
use rs_merkle_tree::stores::MemoryStore;
#[cfg(feature = "rocksdb_store")]
use rs_merkle_tree::stores::RocksDbStore;
#[cfg(feature = "sled_store")]
use rs_merkle_tree::stores::SledStore;
#[cfg(feature = "sqlite_store")]
use rs_merkle_tree::stores::SqliteStore;

#[test]
fn test_stores() {
    // Test all implemented stores
    let mut stores: Vec<Box<dyn Store>> = Vec::new();

    #[cfg(feature = "memory_store")]
    stores.push(Box::new(MemoryStore::new()));
    #[cfg(feature = "sled_store")]
    stores.push(Box::new(SledStore::new("sled.db", true)));
    #[cfg(feature = "sqlite_store")]
    stores.push(Box::new(SqliteStore::new("sqlite.db")));
    #[cfg(feature = "rocksdb_store")]
    stores.push(Box::new(RocksDbStore::new("rocksdb.db")));

    for mut store in stores {
        store.put(&[(0, 0, Node::ZERO)]).unwrap();
        store.put(&[(0, 1, Node::ZERO)]).unwrap();
        store.put(&[(0, 2, Node::ZERO)]).unwrap();
        store
            .put(&[(
                0,
                3,
                to_node!("0x1230000000000000000000000000000000000000000000000000000000000000"),
            )])
            .unwrap();

        assert_eq!(store.get_num_leaves(), 4);
        assert_eq!(store.get(0, 0).unwrap(), Some(Node::ZERO));
        assert_eq!(store.get(0, 1).unwrap(), Some(Node::ZERO));
        assert_eq!(store.get(0, 2).unwrap(), Some(Node::ZERO));
        assert_eq!(
            store.get(0, 3).unwrap(),
            Some(to_node!(
                "0x1230000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }
}
