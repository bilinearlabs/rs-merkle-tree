# rs-merkle-tree

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/bilinearlabs/rs-merkle-tree/rust_main_ci.yml?style=flat-square)
![Codecov (with branch)](https://img.shields.io/codecov/c/github/bilinearlabs/rs-merkle-tree/main?token=1PIHE7U7XQ&style=flat-square)
![GitHub License](https://img.shields.io/github/license/bilinearlabs/rs-merkle-tree?style=flat-square)
[![Join our Discord](https://img.shields.io/badge/Discord-5865F2?logo=discord&logoColor=white&style=flat-square)](https://discord.gg/Et8BTnVBZS)

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
| 32 | keccak256 | sqlite | 9.203 |
| 32 | keccak256 | rocksdb | 11.315 |
| 32 | keccak256 | sled | 38.518 |
| 32 | keccak256 | memory | 88.117 |

### `proof` time

| Depth | Hash | Store | Time |
|---|---|---|---|
| 32 | keccak256 | memory | 880.810 ns |
| 32 | keccak256 | sled | 8.613 µs |
| 32 | keccak256 | rocksdb | 64.176 µs |
| 32 | keccak256 | sqlite | 92.422 µs |

## License

[MIT License](https://github.com/bilinearlabs/rs-merkle-tree/blob/main/LICENSE)
