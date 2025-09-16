use rs_merkle_tree::{
    node::Node,
    store::{MemoryStore, SledStore, SqliteStore, Store},
    to_node,
};
use std::fs;

#[test]
fn test_stores() {
    // Test all implemented stores
    let stores: Vec<Box<dyn Store>> = vec![
        Box::new(MemoryStore::new()),
        Box::new(SledStore::new("sled.db", true)),
        Box::new(SqliteStore::new("sqlite.db")),
    ];

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
