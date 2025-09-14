use rs_merkle_tree::hasher::Keccak256Hasher;
use rs_merkle_tree::store::{MemoryStore, SledStore, Store};
use rs_merkle_tree::tree::{GenericMerkleTree, MerkleTree32};
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
    let mut tree: MerkleTree32 = GenericMerkleTree::new(Keccak256Hasher, MemoryStore::new());

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
    let mut tree: MerkleTree32 = GenericMerkleTree::new(Keccak256Hasher, MemoryStore::new());

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
fn test_disk_space() {
    // Not a test per sec but benchmarks the size of a tree of depth 32 and 1M leaves.
    fs::remove_dir_all("sled.db").ok();

    let num_batches = 1_000;
    let batch_size = 1_000;

    let mut tree: GenericMerkleTree<Keccak256Hasher, SledStore, 32> =
        GenericMerkleTree::new(Keccak256Hasher, SledStore::new("sled.db", false));

    for _ in 0..num_batches {
        let leaves: Vec<Node> = (0..batch_size).map(|_| Node::random()).collect();
        tree.add_leaves(&leaves).unwrap();
    }

    // Show the size of sled.db
    let size_bytes = dir_size(Path::new("sled.db"));
    println!(
        "tree with depth 32 and leaves {} sled.db size: {} bytes ({:.2} MiB)",
        num_batches * batch_size,
        size_bytes,
        size_bytes as f64 / (1024.0 * 1024.0)
    );
}
