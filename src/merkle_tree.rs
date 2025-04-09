use std::marker::PhantomData;
use crate::MtArityHasher as ArityHasher;
use crate::MtDataHasher as DataHasher;

fn length_in_base(mut n: usize, base: usize) -> u32 {
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

/// `Mt` stands for `MerkleTree`
#[derive(Clone, Copy, Debug)]
pub struct MtLvl<'mt_ref, Hash, const ARITY: usize> {
    lvl: Option<&'mt_ref Vec<Hash>>,
}
impl<'mt_ref, Hash, const ARITY: usize> MtLvl<'mt_ref, Hash, ARITY> {
    pub fn new_empty() -> Self {
        Self { lvl: None }
    }
    pub fn new(lvl: &'mt_ref Vec<Hash>) -> Self {
        let lvl = (!lvl.is_empty()).then_some(lvl);
        Self { lvl }
    }

    pub fn is_empty(&self) -> bool {
        self.lvl.is_none()
    }
    /// # panic
    /// * if `self.is_empty()`
    pub fn to_vec(&self) -> &Vec<Hash> {
        self.lvl.unwrap()
    }
}
impl<'mt_ref, Hash: Eq, const ARITY: usize> Eq for MtLvl<'mt_ref, Hash, ARITY> { }
impl<'mt_ref, Hash: Eq, const ARITY: usize> PartialEq for MtLvl<'mt_ref, Hash, ARITY> {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = match (self.is_empty(), other.is_empty()) {
            (true, true) => return true,
            (false, false) => (self.to_vec(), other.to_vec()),
            _ => return false,
        };

        let (a, b) = if a.len() <= b.len() {
            (a, b)
        } else {
            (b, a)
        };
        let a_len = a.len();
        let b_len = b.len();

        if a_len != b_len {
            println!("LEN: {} & {}", a.len(), b.len());
            // in this case tree have different height
            let arity_len = length_in_base(a_len - 1, ARITY);
            if arity_len != length_in_base(b_len - 1, ARITY) { return false }
    
            // tests if all excess elems in b are agree with elemnts in a
            //
            // elements agrees if elements in corresp. window is equal:
            // || 0 1 2 | 3 4 _ | _ _ _ || 
            // the same as:
            // || 0 1 2 | 3 4 4 | 3 4 4 ||
            // window size is calculated by inversing ARITY numeral system numbers in index:
            // index = 12 = 110_3 then repetiotins will be: 222_3 - 110_3 = 112_3 
            // so there will be [2, 1, 1] repetions with window size [1, 3, 9]
            let mut a_index = a_len - 1;
            let mut b_index = a_len;
            let mut window_sz = 1;
            'excess: while a_index != 0 {
                let repetitions = a_index % ARITY;
                for i in 1..=repetitions {
                    let a_index_pre_calc = b_index - i * window_sz;
                    let b_index_pre_calc = b_index;
                    for j in 0..window_sz {
                        let l_index = b_index_pre_calc + j;
                        let r_index = a_index_pre_calc + j;
                        println!("{} =?= {}", l_index, r_index);
                        if b[l_index] != b[r_index] { return false }
                        b_index += 1;
                        if b_index == b_len { break 'excess }
                    }
                }
                window_sz *= ARITY;
                a_index /= ARITY;
            }

            // it can be done faster (less elements can be checked)
            //
            // now catch cases like this:
            // || 0 1 2 | 3 4 _ | _ _ _ || 
            //  not eq
            // || 0 1 2 | 3 4 4 | 3 _ _ ||

            // WIP : STOP HERE
        }

        for i in 0..a_len {
            if a[i] != b[i] { return false }
        }

        return true
    }
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
impl<Hash: Eq, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> Eq for MerkleTree<Hash, Hasher, ARITY> { }
impl<Hash: Eq, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> PartialEq for MerkleTree<Hash, Hasher, ARITY> {
    /// ⚠️ tests only equality of trees itself
    /// (don't test that the hashers are the same, in case if hasher rely on some private arg)
    fn eq(&self, b: &Self) -> bool {
        let a = self; 
        if a.height() != b.height() { println!("TODO:DEL:height"); return false }

        for lvl in (0..a.height()).rev() {
            if a.get_lvl(lvl) != b.get_lvl(lvl) { println!("TODO:DEL:lvl={lvl}"); return false; }
        }

        return true;
    }
}
// TODO: write batched
// TODO: merge
// TODO: split (need clone for Hasher & Hash)

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
    pub fn get_lvl(&self, lvl: usize) -> MtLvl<'_, Hash, ARITY> {
        if lvl < self.height() {
            MtLvl::new(&self.tree_lvls[lvl])
        } else {
            MtLvl::new_empty()
        }
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
    pub fn hash_data<Data>(&mut self, data: Data) -> Hash
    where Hasher: DataHasher<Hash, Data>
    {
        self.hasher.hash_data(data)
    }

    pub fn push_data<Data>(&mut self, data: Data) -> LeafId
    where Hasher: DataHasher<Hash, Data>
    {
        let hash = self.hash_data(data);
        self.push(hash)
    }
}


#[cfg(test)]
#[test]
fn todo_del() {
    assert_eq!(length_in_base(0, 3), 0);
    assert_eq!(length_in_base(1, 3), 1);
    assert_eq!(length_in_base(2, 3), 1);
    assert_eq!(length_in_base(3, 3), 2);
    assert_eq!(length_in_base(5, 3), 2);
    assert_eq!(length_in_base(8, 3), 2);
    assert_eq!(length_in_base(9, 3), 3);
    assert_eq!(length_in_base(10, 3), 3);
    assert_eq!(length_in_base(26, 3), 3);
    assert_eq!(length_in_base(27, 3), 4);
    assert_eq!(length_in_base(28, 3), 4);
    assert_eq!(length_in_base(80, 3), 4);
    assert_eq!(length_in_base(85, 3), 5);
}
