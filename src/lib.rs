mod merkle_tree;
mod hasher;

pub mod utility;

#[cfg(test)]
mod tests;

#[cfg(feature = "bitcoin")]
pub mod bitcoin;

pub use merkle_tree::{MtLvl, LeafId, NodeId};
pub use merkle_tree::{MtProofRef, MtProof};
pub use merkle_tree::{MerkleTree, MerkleBinTree, MerkleTrinaryTree};

pub use hasher::{MtHasher, MtArityHasher, MtDataHasher, MtDataHasherStatic};

#[cfg(any(feature = "unsecure", test))]
pub use hasher::UnsecureHasher;

pub mod prelude {
    pub use crate::{MerkleTree, MerkleBinTree, MerkleTrinaryTree};
    pub use crate::MtProofRef;
    pub use crate::{LeafId, NodeId};
    pub use crate::{MtArityHasher, MtDataHasher};
}