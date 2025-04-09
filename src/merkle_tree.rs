use std::marker::PhantomData;
use crate::MtArityHasher as ArityHasher;
use crate::MtDataHasher as DataHasher;

fn length_in_base(mut n: u64, base: u64) -> usize {
    let mut len = 0;
    while n > 0 {
        n /= base;
        len += 1;
    }
    len
}

struct MerkleTreePath {
    bin_path: Vec<u8>, // TODO: SmallVec
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct LeafId(usize);
impl LeafId {
    #[inline(always)]
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct MerkleTree<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> {
    tree_lvls: Vec<Vec<Hash>>,
    hasher: Box<Hasher>,

    add_lvl_sz: usize,
    new_lvl_cap: usize,
    phantom: PhantomData<Hasher>,
}
// TODO: write batched

impl<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    // TODO: new_capacity / new_height
    pub fn new_minimal(hasher: Hasher) -> Self {
        assert!(ARITY > 1, "`MerkleTree` is a tree, so `ARITY` must be more than 1");
        Self {
            tree_lvls: vec![vec![]],
            hasher: Box::new(hasher),
            add_lvl_sz: 1,
            new_lvl_cap: 1,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree_lvls[0].is_empty()
    }
    #[inline]
    pub fn height(&self) -> usize {
        self.tree_lvls.len()
    }
    #[inline]
    fn leaf_elems(&self) -> usize {
        self.tree_lvls[0].len()
    }

    // TODO: TreeLvl struct that allow to test eq, and can be true if all last is eq to another 
    pub fn get_lvl(&self, lvl: usize) -> &Vec<Hash> {
        &self.tree_lvls[lvl]
    }

    pub fn valid_leaf_id(&self, id: LeafId) -> bool {
        id.0 < self.leaf_elems()
    }
}
impl<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    fn recalc_hashes(&mut self, mut elem_n: usize) {
        for lvl in 1..self.height() {
            let from = elem_n - (elem_n % ARITY);
            for i in 0..ARITY {
                if let Some(hash) = self.tree_lvls[lvl - 1].get(from + i) {
                    self.hasher.hash_arity_one_ref(hash);
                } else {
                    for _ in i..ARITY {
                        let hash = self.tree_lvls[lvl - 1].get(from + i - 1).unwrap();
                        self.hasher.hash_arity_one_ref(hash);
                    }
                    break;
                }
            }
            let new_hash = self.hasher.finish_arity();

            elem_n = elem_n / ARITY;
            if let Some(prev_hash) = self.tree_lvls[lvl].get_mut(elem_n) {
                *prev_hash = new_hash;
            } else {
                self.tree_lvls[lvl].push(new_hash);
            }
        }
    }

    pub fn push(&mut self, hash: Hash) -> LeafId {
        if self.leaf_elems() == self.add_lvl_sz {
            self.add_lvl_sz *= ARITY;
            self.tree_lvls.push(Vec::with_capacity(self.new_lvl_cap));
        }

        let elem_n = self.leaf_elems();
        self.tree_lvls[0].push(hash);
        self.recalc_hashes(elem_n);

        LeafId::new(self.tree_lvls[0].len() - 1)
    }

    /// # panic
    /// * if `self.valid_leaf_id(id)` is false
    pub fn replace(&mut self, hash: Hash, id: LeafId) {
        let elem_n = id.0;
        self.tree_lvls[0][elem_n] = hash;
        self.recalc_hashes(elem_n);
    }

}
impl<Hash, Hasher:  ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY>
{
    pub fn push_data<Data>(&mut self, data: Data) -> LeafId
    where Hasher: DataHasher<Hash, Data>
    {
        let hash = self.hasher.hash_data(data);
        self.push(hash)
    }
}

