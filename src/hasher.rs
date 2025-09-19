use crate::node::Node;
use tiny_keccak::{Hasher, Keccak};

pub trait TreeHasher {
    fn hash(&self, left: &Node, right: &Node) -> Node;
}

// Implements the keccak256 hash function.
pub struct Keccak256Hasher;
impl TreeHasher for Keccak256Hasher {
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

#[cfg(test)]
mod tests {
    use crate::to_node;

    use super::*;

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
}
