# rs-merkle-tree

Merkle tree implementation in Rust with configurable storage backends and hash functions. This Merkle tree implementation features:
* Fixed depth: All proofs have a constant size equal to the `Depth`.
* Append-only: Leaves are added sequentially starting at index `0`. Once added, a leaf cannot be modified.
* Optimized for Merkle proof retrieval: Intermediate leaves are stored so that Merkle proofs can be fetched from memory without needing to be calculated lazily, resulting in very fast retrieval times.


Add `rs-merkle-tree` as a dependency to your Rust `Cargo.toml`.

```toml
[dependencies]
rs-merkle-tree = { git = "https://github.com/bilinearlabs/rs-merkle-tree.git" }
```

You can create a Merkle tree, add leaves, get the number of leaves and get the Merkle proof of a given index as follows. This creates a simple merkle tree using keccak256 hashing algorithm, a memory storage and a depth 32.

```rust
use rs_merkle_tree::to_node;
use rs_merkle_tree::tree::MerkleTree32;

fn main() {
    let mut tree = MerkleTree32::default();
    tree.add_leaves(&[to_node!(
        "0x532c79f3ea0f4873946d1b14770eaa1c157255a003e73da987b858cc287b0482"
    )])
    .unwrap();

    println!("root: {:?}", tree.root().unwrap());
    println!("num leaves: {:?}", tree.num_leaves());
    println!("proof: {:?}", tree.proof(0).unwrap().proof);
}
```

You can customize your tree by choosing a different store, hash function, and depth as follows. This tree also uses keccak256 but persists the leaves in a key-value sled store and has a depth of 32.

```rust
use rs_merkle_tree::store::SledStore;
use rs_merkle_tree::tree::MerkleTree;
use rs_merkle_tree::hasher::Keccak256Hasher;

fn main() {
    let mut tree: MerkleTree<Keccak256Hasher, SledStore, 32> =
        MerkleTree::new(Keccak256Hasher, SledStore::new("sled.db", true));
}
```

## Stores

The following stores are supported:
* [rusqlite](https://github.com/rusqlite/rusqlite)
* [rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)
* [sled](github.com/spacejam/sled)

## Hash functions

The following hash functions are supported:
* [keccak256](https://github.com/debris/tiny-keccak)
* [Poseidon BN254 Circom T3](https://github.com/Lightprotocol/light-poseidon/)

## Benchmarks

The following benchmarks measure in a MacBook Pro the following:
* Consumed disk size
* Leaf insertion throughput in thousands per second.
* Merkle proof generation times.

You can run them with
```
cargo bench --features=all
```

And you can generate the following table with this.
```
python benchmarks.py
```

### Disk space usage

| Store | Depth | Leaves | Size (MiB) |
|---|---|---|---|
| sled | 32 | 1000000 | 290.00 |
| sqlite | 32 | 1000000 | 159.18 |
| rocksdb | 32 | 1000000 | 183.27 |

### `add_leaves` throughput

| Depth | Hash | Store | Throughput (Kelem/s) |
|---|---|---|---|
| 32 | keccak256 | sqlite | 10.382 |
| 32 | keccak256 | rocksdb | 13.145 |
| 32 | keccak256 | sled | 45.265 |
| 32 | keccak256 | memory | 92.030 |

### `proof` time

| Depth | Hash | Store | Time (ms) |
|---|---|---|---|
| 32 | keccak256 | memory | 7.580 |
| 32 | keccak256 | sled | 34.228 |
| 32 | keccak256 | rocksdb | 684.170 |
| 32 | keccak256 | sqlite | 778.840 |

## License

[MIT License](https://github.com/bilinearlabs/rs-merkle-tree/blob/main/LICENSE)