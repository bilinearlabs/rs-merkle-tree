use crate::node::Node;
use alloy::primitives::keccak256;

pub trait TreeHasher {
    fn hash(&self, left: &Node, right: &Node) -> Node;
}

// Implements the keccak256 hash function.
pub struct Keccak256Hasher;
impl TreeHasher for Keccak256Hasher {
    fn hash(&self, left: &Node, right: &Node) -> Node {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(left.as_ref());
        buf[32..].copy_from_slice(right.as_ref());
        Node::from(keccak256(buf).0)
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
