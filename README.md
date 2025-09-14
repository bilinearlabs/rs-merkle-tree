# rs-merkle-tree

Merkle tree implementation in Rust with configurable storage backends and hash functions.


Add `rs-merkle-tree` as a dependency to your Rust `Cargo.toml`.

```toml
[dependencies]
rs-merkle-tree = { git = "https://github.com/bilinearlabs/rs-merkle-tree.git" }
```

You can create a Merkle tree as follows. See also how you can add leaves, get the number of leaves and get the Merkle proof of a given index.

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

TODO: Document how to create a tree with disk persistence.

TODO: Available stores

TODO: Available hash functions

## Benchmarks

TODO:
