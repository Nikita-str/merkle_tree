
mod hash;
pub use hash::{Hash, BitcoinHasher};

#[cfg(all(test, feature = "bitcoin_test"))]
mod tests;

pub type MerkleTreeBitcoin = crate::MerkleTree::<Hash, BitcoinHasher, 2>;
impl MerkleTreeBitcoin {
    // default_from_...
    pub fn new_by_leafs<I>(leafs_iter: I) -> Self
    where I: IntoIterator<Item = Hash>
    {
        Self::new_from_leafs(BitcoinHasher::new(), leafs_iter)
    }
}


#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SingleBlock {
    pub hash: Hash,
    pub mrkl_root: Hash,
    pub nonce: u32,
    #[cfg_attr(feature = "serde", serde(rename = "tx"))]
    pub txs: Vec<SingleTransaction>
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct SingleTransaction {
    pub hash: Hash,
}
