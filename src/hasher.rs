use crate::node::Node;

use ark_bn254::Fr;

use light_poseidon::{Poseidon, PoseidonBytesHasher};
use tiny_keccak::{Hasher as KeccakHasher, Keccak};

pub trait Hasher {
    fn hash(&self, left: &Node, right: &Node) -> Node;
}

// Implements the keccak256 hash function.
pub struct Keccak256Hasher;
impl Hasher for Keccak256Hasher {
    fn hash(&self, left: &Node, right: &Node) -> Node {
        // TODO: Don't instantiate a new keccak for each hash.
        let mut keccak = Keccak::v256();
        keccak.update(left.as_ref());
        keccak.update(right.as_ref());
        let mut buf = [0u8; 32];
        keccak.finalize(&mut buf);
        Node::from(buf)
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
