use crate::errors::MerkleError;
use crate::hasher::Keccak256Hasher;
use crate::hasher::TreeHasher;
use crate::store::MemoryStore;
use crate::store::Store;
use crate::{node::Node, to_node};
use core::ops::Index;
use std::collections::HashMap;
use std::result::Result;

pub const MAX_DEPTH: usize = 32;

pub struct MerkleProof<const Depth: usize> {
    pub proof: [Node; Depth],
    pub leaf: Node,
    pub index: u64,
    pub root: Node,
}

pub struct GenericMerkleTree<H, S, const Depth: usize>
where
    H: TreeHasher,
    S: Store,
{
    hasher: H,
    store: S,
    zeros: Zeros<Depth>,
}

// Type alias for common configuration
pub type MerkleTree32 = GenericMerkleTree<Keccak256Hasher, MemoryStore, 32>;

// Default tree with common configuration
impl Default for MerkleTree32 {
    fn default() -> Self {
        Self::new(Keccak256Hasher, MemoryStore::new())
    }
}

pub struct Zeros<const Depth: usize> {
    front: [Node; Depth],
    last: Node,
}

// TODO: Maybe use "typenum" crate to avoid this.
impl<const Depth: usize> Index<usize> for Zeros<Depth> {
    type Output = Node;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if index < Depth {
            &self.front[index]
        } else if index == Depth {
            &self.last
        } else {
            panic!("index out of bounds");
        }
    }
}

// TODO: Implement send and sync so that the tree can be used in a concurrent context

impl<H, S, const Depth: usize> GenericMerkleTree<H, S, Depth>
where
    H: TreeHasher,
    S: Store,
{
    pub fn new(hasher: H, store: S) -> Self {
        // TODO: Protect from overflow. Eg if depth is 256, then it will overflow.
        // Set a limit, maybe no more than 64?
        let mut zero = [Node::ZERO; Depth];
        for i in 1..Depth {
            zero[i] = hasher.hash(&zero[i - 1], &zero[i - 1]);
        }
        let zeros = Zeros {
            front: zero,
            last: hasher.hash(&zero[Depth - 1], &zero[Depth - 1]),
        };
        Self {
            hasher,
            store,
            zeros,
        }
    }

    pub fn add_leaves(&mut self, leaves: &[Node]) -> Result<(), MerkleError> {
        // Early return
        if leaves.is_empty() {
            return Ok(());
        }

        // Error if leaves do not fit in the tree
        // TODO: Avoid calculating this. Calculate it at init or do the shifting with the generic.
        if self.store.get_num_leaves() + leaves.len() as u64 > (1 << Depth as u64) {
            return Err(MerkleError::TreeFull {
                depth: Depth as u32,
                capacity: 1 << Depth as u64,
            }
            .into());
        }

        // Stores the levels and hashes to be written in a single batch.
        // This allows to batch all writes in a single batch transaction.
        let mut batch: Vec<(u32, u64, Node)> = Vec::with_capacity(leaves.len() * (Depth + 1));

        // Cache for nodes generated in this batch so we can reuse them
        let mut cache: HashMap<(u32, u64), Node> = HashMap::new();

        for (offset, leaf) in leaves.iter().enumerate() {
            let mut idx = self.store.get_num_leaves() + offset as u64;
            let mut h = *leaf;

            // Store the leaf
            batch.push((0, idx, h));
            cache.insert((0, idx), h);

            // Hash up to the root
            for level in 0..Depth {
                let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };

                // If cache hit
                let sib_hash = if let Some(val) = cache.get(&(level as u32, sibling_idx)) {
                    *val
                } else {
                    self.store
                        .get(level as u32, sibling_idx)?
                        .unwrap_or(self.zeros[level])
                };

                let (left, right) = if idx % 2 == 0 {
                    (h, sib_hash)
                } else {
                    (sib_hash, h)
                };

                h = self.hasher.hash(&left, &right);
                idx /= 2;

                batch.push(((level + 1) as u32, idx, h));
                cache.insert(((level + 1) as u32, idx), h);
            }
        }

        // Update all values in a single batch
        self.store.put(&batch)?;

        Ok(())
    }

    pub fn root(&self) -> Result<Node, MerkleError> {
        Ok(self
            .store
            .get(Depth as u32, 0)?
            .unwrap_or(self.zeros[Depth]))
    }

    pub fn proof(&self, leaf_idx: u64) -> Result<MerkleProof<Depth>, MerkleError> {
        // Implementation detail. Allow proofs even beyond the number of leaves.
        // Since it has fixed depth it is technically correct.
        // Error if leaf_idx is out of bounds.
        // if leaf_idx >= self.store.get_num_leaves() {
        //    return Err(MerkleError::LeafIndexOutOfBounds {
        //        index: leaf_idx,
        //        num_leaves: self.store.get_num_leaves(),
        //    }
        //    .into());
        //}

        if leaf_idx > 1 << Depth as u64 {
            return Err(MerkleError::LeafIndexOutOfBounds {
                index: leaf_idx,
                num_leaves: 1 << Depth as u64,
            }
            .into());
        }

        let mut proof = [Node::ZERO; Depth];
        let mut idx = leaf_idx;

        // Go up the root taking siblings as we go. Siblings are taken from:
        // - The store
        // - Or zeros
        for depth in 0..Depth {
            let sibling = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sib_hash = self
                .store
                .get(depth as u32, sibling)?
                .unwrap_or(self.zeros[depth]);
            proof[depth] = sib_hash;
            idx /= 2;
        }

        Ok(MerkleProof::<Depth> {
            proof,
            leaf: self.store.get(0, leaf_idx)?.unwrap_or(self.zeros[0]),
            index: leaf_idx,
            root: self.root()?,
        })
    }

    pub fn verify_proof(&self, proof: &MerkleProof<Depth>) -> Result<bool, MerkleError> {
        let mut computed_hash = proof.leaf;
        for (j, sibling_hash) in proof.proof.iter().enumerate() {
            let (left, right) = if proof.index & (1 << j) == 0 {
                (computed_hash, *sibling_hash)
            } else {
                (*sibling_hash, computed_hash)
            };
            computed_hash = self.hasher.hash(&left, &right);
        }
        Ok(computed_hash == proof.root)
    }

    pub fn num_leaves(&self) -> u64 {
        self.store.get_num_leaves()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_keccak_32() {
        let hasher = Keccak256Hasher;
        let store = MemoryStore::new();
        let tree: MerkleTree32 = GenericMerkleTree::new(hasher, store);

        // Test vector of expected zeros at each level.
        // Depth: 32
        // Hashing: Keccak256
        let expected_zeros = [
            to_node!("0x0000000000000000000000000000000000000000000000000000000000000000"),
            to_node!("0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5"),
            to_node!("0xb4c11951957c6f8f642c4af61cd6b24640fec6dc7fc607ee8206a99e92410d30"),
            to_node!("0x21ddb9a356815c3fac1026b6dec5df3124afbadb485c9ba5a3e3398a04b7ba85"),
            to_node!("0xe58769b32a1beaf1ea27375a44095a0d1fb664ce2dd358e7fcbfb78c26a19344"),
            to_node!("0x0eb01ebfc9ed27500cd4dfc979272d1f0913cc9f66540d7e8005811109e1cf2d"),
            to_node!("0x887c22bd8750d34016ac3c66b5ff102dacdd73f6b014e710b51e8022af9a1968"),
            to_node!("0xffd70157e48063fc33c97a050f7f640233bf646cc98d9524c6b92bcf3ab56f83"),
            to_node!("0x9867cc5f7f196b93bae1e27e6320742445d290f2263827498b54fec539f756af"),
            to_node!("0xcefad4e508c098b9a7e1d8feb19955fb02ba9675585078710969d3440f5054e0"),
            to_node!("0xf9dc3e7fe016e050eff260334f18a5d4fe391d82092319f5964f2e2eb7c1c3a5"),
            to_node!("0xf8b13a49e282f609c317a833fb8d976d11517c571d1221a265d25af778ecf892"),
            to_node!("0x3490c6ceeb450aecdc82e28293031d10c7d73bf85e57bf041a97360aa2c5d99c"),
            to_node!("0xc1df82d9c4b87413eae2ef048f94b4d3554cea73d92b0f7af96e0271c691e2bb"),
            to_node!("0x5c67add7c6caf302256adedf7ab114da0acfe870d449a3a489f781d659e8becc"),
            to_node!("0xda7bce9f4e8618b6bd2f4132ce798cdc7a60e7e1460a7299e3c6342a579626d2"),
            to_node!("0x2733e50f526ec2fa19a22b31e8ed50f23cd1fdf94c9154ed3a7609a2f1ff981f"),
            to_node!("0xe1d3b5c807b281e4683cc6d6315cf95b9ade8641defcb32372f1c126e398ef7a"),
            to_node!("0x5a2dce0a8a7f68bb74560f8f71837c2c2ebbcbf7fffb42ae1896f13f7c7479a0"),
            to_node!("0xb46a28b6f55540f89444f63de0378e3d121be09e06cc9ded1c20e65876d36aa0"),
            to_node!("0xc65e9645644786b620e2dd2ad648ddfcbf4a7e5b1a3a4ecfe7f64667a3f0b7e2"),
            to_node!("0xf4418588ed35a2458cffeb39b93d26f18d2ab13bdce6aee58e7b99359ec2dfd9"),
            to_node!("0x5a9c16dc00d6ef18b7933a6f8dc65ccb55667138776f7dea101070dc8796e377"),
            to_node!("0x4df84f40ae0c8229d0d6069e5c8f39a7c299677a09d367fc7b05e3bc380ee652"),
            to_node!("0xcdc72595f74c7b1043d0e1ffbab734648c838dfb0527d971b602bc216c9619ef"),
            to_node!("0x0abf5ac974a1ed57f4050aa510dd9c74f508277b39d7973bb2dfccc5eeb0618d"),
            to_node!("0xb8cd74046ff337f0a7bf2c8e03e10f642c1886798d71806ab1e888d9e5ee87d0"),
            to_node!("0x838c5655cb21c6cb83313b5a631175dff4963772cce9108188b34ac87c81c41e"),
            to_node!("0x662ee4dd2dd7b2bc707961b1e646c4047669dcb6584f0d8d770daf5d7e7deb2e"),
            to_node!("0x388ab20e2573d171a88108e79d820e98f26c0b84aa8b2f4aa4968dbb818ea322"),
            to_node!("0x93237c50ba75ee485f4c22adf2f741400bdf8d6a9cc7df7ecae576221665d735"),
            to_node!("0x8448818bb4ae4562849e949e17ac16e0be16688e156b5cf15e098c627c0056a9"),
            to_node!("0x27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"),
        ];

        for (i, zero) in tree.zeros.front.iter().enumerate() {
            assert_eq!(zero, &expected_zeros[i]);
        }
        assert_eq!(tree.zeros.last, expected_zeros[32]);
    }

    #[test]
    fn test_tree_full_error() {
        let hasher = Keccak256Hasher;
        let store = MemoryStore::new();
        let mut tree = GenericMerkleTree::<Keccak256Hasher, MemoryStore, 3>::new(hasher, store);

        tree.add_leaves(&(0..8).map(|_| Node::ZERO).collect::<Vec<Node>>())
            .unwrap();

        // It errors since the tree is full
        assert!(tree.add_leaves(&[Node::ZERO]).is_err());
    }
}
