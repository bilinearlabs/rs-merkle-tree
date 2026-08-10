use crate::node::Node;

use ark_bn254::Fr;

use keccak_batch::{keccak256, keccak256_many_into};
use light_poseidon::{Poseidon, PoseidonBytesHasher};

pub trait Hasher {
    fn hash(&self, left: &Node, right: &Node) -> Node;

    /// Hashes `pairs[i]` into `out[i]`; `pairs` and `out` must be equally long.
    /// This is a default implementation. If your hashing implementation doesnt support
    /// vectorization, there is no need to implement this. If it does (see keccak)
    /// you can optionally implement this, which hashes pairs faster than the naive implementation.
    fn hash_pairs(&self, pairs: &[[Node; 2]], out: &mut [Node]) {
        assert_eq!(pairs.len(), out.len(), "pairs and out must be equally long");
        for (parent, [left, right]) in out.iter_mut().zip(pairs) {
            *parent = self.hash(left, right);
        }
    }
}

/// One `left || right` message, the only shape this tree ever hashes.
const MESSAGE: usize = 2 * Node::LEN;

/// This is the biggest batch we pass to `keccak256_many_into`, but
/// note that the function detects the widest backend the
/// running CPU supports and splits each batch across it.
/// This is set to 8 since is the widest batch any backend consumes, AVX-512
/// running eight 64-bit lanes.
/// This allows to not allocate memory in the heap, since size is known at compile
/// time. But note that this number acts as an upper limit. A platform supporting only
/// 4 operations in paralel, will pass `keccak256_many_into` with 8 hashes, but unthe the hood
/// it will be split in 2 batches.
const STAGE: usize = 8;

pub struct Keccak256Hasher;

impl Hasher for Keccak256Hasher {
    fn hash(&self, left: &Node, right: &Node) -> Node {
        let mut message = [0u8; MESSAGE];
        message[..Node::LEN].copy_from_slice(left.as_ref());
        message[Node::LEN..].copy_from_slice(right.as_ref());
        Node::from(keccak256(message))
    }

    fn hash_pairs(&self, pairs: &[[Node; 2]], out: &mut [Node]) {
        assert_eq!(pairs.len(), out.len(), "pairs and out must be equally long");

        // For each STAGE batch
        for (group, parents) in pairs.chunks(STAGE).zip(out.chunks_mut(STAGE)) {
            // We need to convert to &[&[u8]], which is what keccak256_many_into
            // accepts.
            let mut staged: [&[u8]; STAGE] = [&[]; STAGE];
            for (slot, pair) in staged.iter_mut().zip(group) {
                *slot = Node::as_bytes(pair);
            }

            // Trimming to the run length lets the last batch be short: a
            // partial batch is split across the narrower widths, not dropped
            // to a scalar tail.
            let mut digests = [[0u8; Node::LEN]; STAGE];
            keccak256_many_into(&staged[..group.len()], &mut digests[..group.len()]);

            for (parent, digest) in parents.iter_mut().zip(digests) {
                *parent = Node::from(digest);
            }
        }
    }
}

// Implements the circom-compatible Poseidon hash function (T=3)
pub struct PoseidonHasher;

impl Hasher for PoseidonHasher {
    fn hash(&self, left: &Node, right: &Node) -> Node {
        // circom-compatible Poseidon with 2 inputs (T=3)
        let mut poseidon = Poseidon::<Fr>::new_circom(2).unwrap();

        let res = poseidon
            .hash_bytes_be(&[left.as_ref(), right.as_ref()])
            .unwrap();

        Node::from(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_node;

    #[test]
    fn test_keccak256_hash() {
        let hasher = Keccak256Hasher;
        let result = hasher.hash(
            &to_node!("0x1230000000000000000000000000000000000000000000000000000000000000"),
            &to_node!("0x1230000000000000000000000000000000000000000000000000000000000000"),
        );
        assert_eq!(
            result,
            to_node!("0x760bde345debf3075c7fc0bcd2134e16ce5fc1a13adaa66ec6452a391f70595c")
        );
    }

    /// Batch sizes either side of every SIMD width the crate dispatches to and
    /// of the staging buffer, so full batches, short trailing batches, and the
    /// empty run all reproduce pair-by-pair hashing.
    #[test]
    fn test_keccak256_hash_pairs_matches_hash() {
        let hasher = Keccak256Hasher;
        let pairs: Vec<[Node; 2]> = (0..65).map(|_| [Node::random(), Node::random()]).collect();

        for len in [0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 33, 64, 65] {
            let pairs = &pairs[..len];
            let mut parents = vec![Node::ZERO; len];
            hasher.hash_pairs(pairs, &mut parents);

            for (i, (parent, [left, right])) in parents.iter().zip(pairs).enumerate() {
                assert_eq!(parent, &hasher.hash(left, right), "pair {i} of {len}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "pairs and out must be equally long")]
    fn test_keccak256_hash_pairs_rejects_length_mismatch() {
        Keccak256Hasher.hash_pairs(&[[Node::ZERO, Node::ZERO]], &mut []);
    }

    #[test]
    fn test_poseidon_hash() {
        let hasher = PoseidonHasher;
        let result = hasher.hash(
            &to_node!("0x0000000000000000000000000000000000000000000000000000000000000000"),
            &to_node!("0x0000000000000000000000000000000000000000000000000000000000000000"),
        );

        assert_eq!(
            result,
            to_node!("0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864")
        );
    }
}
