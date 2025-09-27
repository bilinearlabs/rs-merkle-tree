use rs_merkle_tree::hasher::Keccak256Hasher;
use rs_merkle_tree::store::{MemoryStore, RocksDbStore, SledStore, SqliteStore, Store};
use rs_merkle_tree::tree::{MerkleTree, MerkleTree32};
use rs_merkle_tree::{node::Node, to_node};
use std::fs;
use std::path::Path;

fn dir_size(path: &Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            size += dir_size(&entry.path());
        }
    }
    size
}

#[test]
fn test_merkle_tree_keccak_32_memory() {
    let mut tree: MerkleTree32 = MerkleTree::new(Keccak256Hasher, MemoryStore::new());

    // create 10k leaves.
    let leaves = (0..10_000)
        .map(|i| to_node!(format!("0x{:064x}", i).as_str()))
        .collect::<Vec<Node>>();

    // create a tree with 10k leaves.
    for i in &leaves {
        tree.add_leaves(&[*i]).unwrap();
    }

    assert_eq!(tree.num_leaves(), 10_000);
    assert_eq!(
        tree.root().unwrap(),
        to_node!("0x532c79f3ea0f4873946d1b14770eaa1c157255a003e73da987b858cc287b0482")
    );

    // reset the tree.
    let mut tree: MerkleTree32 = MerkleTree::new(Keccak256Hasher, MemoryStore::new());

    // same but add them in batches of 1_000.
    for batch in leaves.chunks(1_000) {
        tree.add_leaves(&batch).unwrap();
    }

    assert_eq!(tree.num_leaves(), 10_000);
    assert_eq!(
        tree.root().unwrap(),
        to_node!("0x532c79f3ea0f4873946d1b14770eaa1c157255a003e73da987b858cc287b0482")
    );

    // Get proofs for each leaf and verify them.
    for i in 0..10_000 {
        let proof = tree.proof(i).unwrap();
        assert_eq!(proof.proof.len(), 32);
        assert_eq!(tree.verify_proof(&proof).unwrap(), true);
    }

    // TODO: Once async is implemented, ensure proofs are always consistent.
}

#[test]
#[ignore = "run it on demand, slow and takes some disk space"]
fn test_disk_space() {
    // Run as: cargo test --release test_disk_space -- --ignored --no-capture
    // Not a unit-test per-se; rather, it benchmarks the on-disk size of a depth-32
    // tree containing one million leaves for each backing store.

    // Insert NUM_BATCHES * BATCH_SIZE leaves.
    const NUM_BATCHES: usize = 1_000;
    const BATCH_SIZE: usize = 1_000;

    // Clean up any previous runs.
    ["sled.db", "sqlite.db", "rocksdb.db"]
        .into_iter()
        .for_each(|p| {
            fs::remove_dir_all(p).ok();
            fs::remove_file(p).ok();
        });

    // Helper that creates the tree with a given store and inserts `NUM_BATCHES * BATCH_SIZE` leaves.
    fn bench_store<S, F>(db_name: &str, new_store: F)
    where
        S: Store,
        F: FnOnce() -> S,
    {
        let mut tree: MerkleTree<Keccak256Hasher, S, 32> =
            MerkleTree::new(Keccak256Hasher, new_store());

        for _ in 0..NUM_BATCHES {
            let leaves: Vec<Node> = (0..BATCH_SIZE).map(|_| Node::random()).collect();
            tree.add_leaves(&leaves).unwrap();
        }

        print_size(db_name, db_name, (NUM_BATCHES * BATCH_SIZE) as u64);
    }

    bench_store::<SledStore, _>("sled.db", || SledStore::new("sled.db", false));
    bench_store::<SqliteStore, _>("sqlite.db", || SqliteStore::new("sqlite.db"));
    bench_store::<RocksDbStore, _>("rocksdb.db", || RocksDbStore::new("rocksdb.db"));
}

fn print_size(name: &str, file: &str, num_leaves: u64) {
    // Hardcoded depth for now.
    let depth = 32;

    let size_bytes = dir_size(Path::new(file));
    println!(
        "store {} depth {} num_leaves {} size: {:.2} MiB",
        name,
        depth,
        num_leaves,
        size_bytes as f64 / (1024.0 * 1024.0)
    );
    fs::remove_dir_all(file).ok();
}
