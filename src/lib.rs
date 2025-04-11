mod merkle_tree;
mod hasher;

pub mod utility;

#[cfg(test)]
mod tests;

pub use merkle_tree::{MerkleTree, MtLvl};
pub use hasher::{MtHasher, MtArityHasher, MtDataHasher, MtDataHasherStatic};

#[cfg(any(feature = "unsecure", test))]
pub use hasher::UnsecureHasher;