use rs_merkle_tree::{
    node::Node,
    store::{MemoryStore, RocksDbStore, SledStore, SqliteStore, Store},
    to_node,
};
use std::fs;
use temp_file::TempFile;

#[test]
fn test_stores() {
    let temp_file_sqlite = TempFile::with_suffix("_sqlite.db").unwrap();
    let path_sqlite = temp_file_sqlite
        .path()
        .as_os_str()
        .to_str()
        .expect("Failed to build path for SQLite");
    println!("SQLite path: {}", path_sqlite);
    let temp_file_rockdb = TempFile::with_suffix("_rocksdb.db").unwrap();
    let path_rocksdb = temp_file_rockdb
        .path()
        .as_os_str()
        .to_str()
        .expect("Failed to build path for RocksDB")
        .to_owned();
    // RocksDB expects the file to not exists, so we make a temp name and force the cleanup of the file.
    temp_file_rockdb
        .cleanup()
        .expect("Failed to cleanup RocksDB");
    println!("RocksDB path: {}", path_rocksdb);

    // Test all implemented stores
    let stores: Vec<Box<dyn Store>> = vec![
        Box::new(MemoryStore::new()),
        Box::new(SledStore::new("/tmp/sled.db", true)),
        Box::new(SqliteStore::new(path_sqlite)),
        Box::new(RocksDbStore::new(&path_rocksdb)),
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

    // Now delete the RocksDB directory.
    fs::remove_dir_all(path_rocksdb).expect("Failed to delete RocksDB file");
}
