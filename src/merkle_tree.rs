use std::marker::PhantomData;
use std::ops::Range;
use crate::utility::{get_pad_index, length_in_base};
use crate::MtArityHasher as ArityHasher;
use crate::MtDataHasher as DataHasher;
use crate::MtDataHasherStatic as StaticDataHasher;

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
    pub fn len(&self) -> usize {
        self.lvl.map(|x|x.len()).unwrap_or(0)
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
impl<'mt_ref, Hash: Clone, const ARITY: usize> MtLvl<'mt_ref, Hash, ARITY> {
    pub fn continuation(&self) -> Option<Vec<Hash>> {
        let Some(lvl) = self.lvl.cloned() else { return None };
        Some(Self::vec_continuation(lvl))
    }
    pub fn vec_continuation(mut lvl: Vec<Hash>) -> Vec<Hash> {
        let mut buf = Vec::new();
        let mut arity_mask = lvl.len() - 1;
        let mut win_sz = 1;
        while arity_mask > 0 {
            let amount_of_win = (ARITY - 1) - (arity_mask % ARITY);
            let cur_len = lvl.len();

            // lvl.copy_within, but you need unsafe null mem alloc
            buf.clear();
            buf.extend_from_slice(&lvl[cur_len - win_sz..]);
            for _ in 0..amount_of_win {
                lvl.extend(buf.iter().cloned());
            }
            
            arity_mask /= ARITY;
            win_sz *= ARITY;
        }
        lvl
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
            // if  arity_lens non equal => tree have different height
            let arity_len = length_in_base(a_len - 1, ARITY);
            if arity_len != length_in_base(b_len - 1, ARITY) { return false }
            assert_ne!(arity_len, 0, "when arity is 0 a_len must be equal b_len");
    
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
                let repetitions = (ARITY - 1) - (a_index % ARITY);
                for i in 1..=repetitions {
                    let r_index = b_index - i * window_sz;
                    let l_index = b_index;

                    for j in 0..window_sz {
                        let l_index = l_index + j;
                        let r_index = r_index + j;

                        if b[l_index] != b[r_index] { return false }
                        
                        b_index += 1;
                        if b_index == b_len { break 'excess }
                    }
                }
                window_sz *= ARITY;
                a_index /= ARITY;
            }


            // ⚠️ it can be done faster:
            //    | less elements can be checked, 
            //    | but for sureness I left it like this
            //    | &
            //    | also (seems like) we can calc `get_pad_index` more effectively 
            //
            // now catch next cases:
            // || 0 1 2 | 3 4 _ | _ _ _ || 
            //  not eq
            // || 0 1 2 | 3 4 4 | 3 _ _ ||
            let padding_sz = ARITY.pow(arity_len - 1);
            if b_len % padding_sz != 0 {
                // we must test padding
                let a_pos_start = ((a_len - 1) / padding_sz) * padding_sz;
                let b_pos_start = ((b_len - 1) / padding_sz) * padding_sz;
                
                for i in 0..padding_sz {
                    let l_index = a_pos_start + i;
                    let r_index = b_pos_start + i;

                    let l_index = get_pad_index(l_index, a_len - 1, ARITY);
                    let r_index = get_pad_index(r_index, b_len - 1, ARITY);
                    
                    if b[l_index] != b[r_index] { return false }
                }
            }
        }

        for i in 0..a_len {
            if a[i] != b[i] { return false }
        }

        return true
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeafId(usize);
impl LeafId {
    #[inline(always)]
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct MerkleTree<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> {
    tree_lvls: Vec<Vec<Hash>>,
    hasher: Box<Hasher>,

    add_lvl_sz: usize,
    new_lvl_cap: usize,
    phantom: PhantomData<Hasher>,
}
// TODO: get lvl
// TODO: split (need clone for Hasher & Hash)

impl<Hash: Eq, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// Test equality of two trees by comparing only equality of height and root.\
    /// In most case it is enough.
    /// 
    /// In most cases, you should wrap `MerkleTree` with concrete generic args & 
    /// impl [`Eq`] trait with suitable eq fn: \
    /// See also [`Self::eq_full`]
    pub fn eq_weak(&self, other: &Self) -> bool {
        if self.is_empty() { return other.is_empty() }
        if !self.hasher.is_the_same(&other.hasher) { return false }

        let height_eq = self.height() == other.height();
        let root_eq = self.root_ref() == other.root_ref();
        height_eq && root_eq
    }
    /// Test non-equality of two trees by comparing only equality of height and root.\
    /// In most case it is enough.
    /// 
    /// # Guarantees
    /// Always the same as `!self.eq_weak(other)` 
    pub fn ne_weak(&self, other: &Self) -> bool {
        !self.eq_weak(other)
    }
    
    /// Test equality of two trees by comparing all levels.\
    /// 
    /// At least needs for tests
    /// 
    /// In most cases, you should wrap `MerkleTree` with concrete generic args & 
    /// impl [`Eq`] trait with suitable eq fn: \
    /// See also [`Self::eq_weak`]
    pub fn eq_full(&self, b: &Self) -> bool {
        let a = self; 
        if a.height() != b.height() { return false }
        if !a.hasher.is_the_same(&b.hasher) { return false }

        for lvl in (0..a.height()).rev() {
            if a.get_lvl(lvl) != b.get_lvl(lvl) { return false; }
        }

        return true;
    }
    /// Test non-equality of two trees by comparing all levels.\
    /// 
    /// # Guarantees
    /// Always the same as `!self.eq_full(other)` 
    pub fn ne_full(&self, other: &Self) -> bool {
        !self.eq_full(other)
    }
}
impl<Hash: Clone, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// # panic
    /// * if `self.is_empty()`
    #[inline]
    pub fn root(&self) -> Hash {
        self.tree_lvls[self.height() - 1][0].clone()
    }
}
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
    pub fn new_from_leafs<I>(hasher: Hasher, leafs_iter: I) -> Self
    where I: IntoIterator<Item = Hash>
    {
        let mut tree = Self::new_minimal(hasher);
        let leafs_batch = leafs_iter.into_iter().collect::<Vec<_>>();
        tree.push_batched(leafs_batch);
        tree
    }

    /// # panic
    /// * if `self.is_empty()`
    #[inline]
    pub fn root_ref(&self) -> &Hash {
        &self.tree_lvls[self.height() - 1][0]
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
    #[inline]
    fn last_leaf_id(&self) -> LeafId {
        LeafId(self.tree_lvls[0].len())
    }

    /// Return requested level of tree.
    /// 
    /// For example:
    /// * Leaf elements have `0`` level
    /// * Root element has `self.height() - 1` level
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
    fn make_lvl_valid(&mut self, lvl: usize, expected_len: usize) {
        if self.tree_lvls.len() <= lvl {
            self.tree_lvls.push(Vec::with_capacity(self.new_lvl_cap.max(expected_len)));
        }
    }

    /// # Input
    /// * `pre_lvl_range`: elemnts of level `lvl - 1` from which hashes calculated
    /// * `lvl`: for which lvl calculate hashes
    /// # Return
    /// * `last_is_even: bool` is last group even
    fn calc_lvl_hashes(&mut self, pre_lvl_range: Range<usize>, lvl: usize) -> bool {
        let (from, to) = (pre_lvl_range.start, pre_lvl_range.end);
        let last_is_even = to % ARITY == 0;

        // in next range all `tree_lvls[lvl - 1]` is valid
        for elem_index in (from / ARITY)..(to / ARITY) {
            for win_index in 0..ARITY {
                self.hasher.hash_arity_one_ref(&self.tree_lvls[lvl - 1][elem_index * ARITY + win_index]);
            }
            let new_hash = self.hasher.finish_arity();
            self.set_or_push(elem_index, lvl, new_hash);
        }

        if !last_is_even {
            let new_hash = self.calc_possibly_uneven_group_hash(to - 1, lvl);
            let elem_index = to / ARITY;
            self.set_or_push(elem_index, lvl, new_hash);
        }

        last_is_even
    }

    /// Calculate hash for group of ARITY elements (that can have less than ARITY child) on level `lvl - 1`.\
    /// Thah hash used on coresp. elem on level `lvl`.
    /// 
    /// ```txt
    /// level [lvl - 1]: hash(A B C D E) --> result. 
    /// Where each from {A, B, C, D, E} is group element  
    /// 
    /// Result used for level[lvl][index_of_elem_from_group / ARITY].
    /// ```
    /// 
    /// # panic
    /// * if `lvl` is `0` 
    fn calc_possibly_uneven_group_hash(&mut self, elem_n_from_group: usize, lvl: usize) -> Hash {
        let group_from = elem_n_from_group - (elem_n_from_group % ARITY);
        for i in 0..ARITY {
            if let Some(hash) = self.tree_lvls[lvl - 1].get(group_from + i) {
                self.hasher.hash_arity_one_ref(hash);
            } else {
                for _ in i..ARITY {
                    let hash = self.tree_lvls[lvl - 1].get(group_from + i - 1).unwrap();
                    self.hasher.hash_arity_one_ref(hash);
                }
                break;
            }
        }
        self.hasher.finish_arity()
    }

    fn set_or_push(&mut self, index: usize, lvl: usize, new_hash: Hash) {
        if let Some(prev_hash) = self.tree_lvls[lvl].get_mut(index) {
            *prev_hash = new_hash;
        } else {
            self.tree_lvls[lvl].push(new_hash);
        }
    }

    fn recalc_elem_hashes(&mut self, mut elem_n: usize) {
        for lvl in 1..self.height() {
            let new_hash = self.calc_possibly_uneven_group_hash(elem_n, lvl);

            elem_n = elem_n / ARITY;
            self.set_or_push(elem_n, lvl, new_hash);
        }
    }

    /// Add a leaf.
    /// 
    /// If you need push data you can use [`Self::push_data`]
    /// 
    /// If you need push many elements, better use 
    /// [`Self::push_batched`] & [`Self::push_batched_data`] 
    /// they are faster.
    pub fn push(&mut self, hash: Hash) -> LeafId {
        if self.leaf_elems() == self.add_lvl_sz {
            self.add_lvl_sz *= ARITY;
            self.tree_lvls.push(Vec::with_capacity(self.new_lvl_cap));
        }

        let elem_n = self.leaf_elems();
        self.tree_lvls[0].push(hash);
        self.recalc_elem_hashes(elem_n);

        LeafId::new(self.tree_lvls[0].len() - 1)
    }

    /// Replace a leaf.
    /// 
    /// If you need replace by data you can use [`Self::replace_data`]
    /// 
    /// If you need push many elements, better use 
    /// [`Self::push_batched`] & [`Self::push_batched_data`] 
    /// they are faster.
    /// 
    /// # panic
    /// * if `self.valid_leaf_id(id)` is false
    pub fn replace(&mut self, hash: Hash, id: LeafId) {
        let elem_n = id.0;
        self.tree_lvls[0][elem_n] = hash;
        self.recalc_elem_hashes(elem_n);
    }

    /// Add batch of leafs by hashing data.\
    /// It's faster than many single pushes.
    /// 
    /// If you need push a single data you can use [`Self::push_data`]
    pub fn push_batched_data<Data>(&mut self, batch: impl IntoIterator<Item = Data>) -> std::ops::Range<LeafId>
    where Hasher: StaticDataHasher<Hash, Data>
    {
        let map = |data|Hasher::hash_data_static(data);
        self.push_batched(batch.into_iter().map(map))
    }

    /// Add batch of leafs.\
    /// It's faster than many single pushes.
    /// 
    /// If you need push a single leaf you can use [`Self::push`]
    /// 
    /// If you need push batch of data you can use [`Self::push_batched_data`]
    pub fn push_batched(&mut self, batch: impl IntoIterator<Item = Hash>) -> std::ops::Range<LeafId> {
        self.replace_batched(batch, self.last_leaf_id())
    }

    /// Replace batch of leafs by hashing data.\
    /// It's faster than many single replaces.
    /// 
    /// If you need replace a single data you can use [`Self::replace_data`]
    pub fn replace_batched_data<Data, I>(&mut self, batch: I, start_id: LeafId) -> std::ops::Range<LeafId>
    where
        I: IntoIterator<Item = Data>,
        Hasher: StaticDataHasher<Hash, Data>,
    {
        let map = |data|Hasher::hash_data_static(data);
        self.replace_batched(batch.into_iter().map(map), start_id)
    }

    /// Replace batch of leafs.\
    /// It's faster than many single replaces.
    /// 
    /// If you need replace a single leaf you can use [`Self::replace`]
    /// 
    /// If you need replace batch of data you can use [`Self::replace_batched_data`]
    /// 
    /// # panic
    /// * if `start_id` > `last_leaf_id`
    pub fn replace_batched(&mut self, batch: impl IntoIterator<Item = Hash>, start_id: LeafId) -> std::ops::Range<LeafId> {
        if start_id > self.last_leaf_id() {
            panic!("invalid `start_id`")
        }

        let from = start_id;
        let mut to = start_id.0;

        let mut ended = false;
        let mut batch = batch.into_iter();
        
        // replacing hashes from batches while they in valid range  
        for index in start_id.0..self.leaf_elems() {
            let Some(next_hash) = batch.next() else {
                ended = true;
                to = index;
                break
            };
            self.tree_lvls[0][index] = next_hash;
        }

        // if batch not ended during replacing -- add rest hashes to the end of leaf level
        if !ended {
            self.tree_lvls[0].extend(batch);
            to = self.tree_lvls[0].len();
        }
        let to = LeafId::new(to);

        // update hashes on next levels:
        if from != to {
            let mut from = from.0;
            let mut to = to.0;
            let mut lvl = 1;
            let lvl_must = length_in_base(self.tree_lvls[0].len() - 1, ARITY) as usize + 1;

            while lvl != lvl_must {
                self.make_lvl_valid(lvl, (to / ARITY - from / ARITY) + 1);
                let last_is_even = self.calc_lvl_hashes(from..to, lvl);
                
                lvl += 1;
                from /= ARITY;
                to = (to / ARITY) + (!last_is_even) as usize;
            }
        }

        from..to
    }

    /// It is effective if leaf amount is `pow(ARITY, exp)` & all `MerkleTree`s have the same len.\
    /// It can be used for parallelism.
    ///  
    /// # returns
    /// * [`None`] if `iter` is empty 
    /// * [`Some`] of merged tree otherwise 
    /// # panic
    /// * if some elements of iterator have non-equal hasher (see [`ArityHasher::is_the_same`]); 
    pub fn new_merged(iter: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut iter = iter.into_iter();
        let Some(mut tree) = iter.next() else { return None };
        tree.merge(iter);
        Some(tree)
    }
    
    /// It is very effective (more effective than `push_batched`) if leaf amount is `pow(ARITY, exp)` 
    /// & all `MerkleTree`s have the same len.\
    /// If the order of leafs is not important (you can say the order afterwards) 
    /// and `MerkleTree`s have different len, then firstly
    /// place big trees that satisfy the two previous conditions. 
    /// 
    /// It can be used for parallelism (accept only trees with some leaf amount, or last trees, if it is time to create tree).
    /// 
    /// # returns
    /// * [`None`] if `iter` is empty 
    /// * [`Some`] of merged tree otherwise
    pub fn merge(&mut self, iter: impl IntoIterator<Item = Self>) {
        for other in iter {
            if other.leaf_elems() == 0 { continue }
            let other_height = other.height();

            let mut recalc_index: Option<usize> = None;
            for (lvl, tree_lvl) in other.tree_lvls.into_iter().enumerate() {
                self.make_lvl_valid(lvl, tree_lvl.len());
                
                if let Some(from_index) = recalc_index {
                    let pre_lvl_range = from_index..self.tree_lvls[lvl - 1].len();
                    self.calc_lvl_hashes(pre_lvl_range, lvl);
                    recalc_index = Some(from_index / ARITY);
                } else {
                    let left_len = self.tree_lvls[lvl].len();
                    self.tree_lvls[lvl].extend(tree_lvl);
                    if left_len % ARITY != 0 {
                        recalc_index = Some(left_len);
                    }
                }
            }

            // now calc top if needed
            let leafs_amount = self.leaf_elems();
            if leafs_amount == 0 { continue }

            let mut recalc_index = recalc_index.unwrap_or(0);
            let lvl_must = length_in_base(leafs_amount - 1, ARITY) as usize + 1;
            for lvl in other_height..lvl_must {
                let pre_lvl_len = self.tree_lvls[lvl - 1].len();
                self.make_lvl_valid(lvl,  pre_lvl_len / ARITY);
                
                let pre_lvl_range = recalc_index..pre_lvl_len;
                self.calc_lvl_hashes(pre_lvl_range, lvl);
                recalc_index = recalc_index / ARITY;
            }
        }
    }

}
impl<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY>
{
    pub fn hash_data_static<Data>(data: Data) -> Hash
    where Hasher: StaticDataHasher<Hash, Data>
    {
        Hasher::hash_data_static(data)
    }

    pub fn hash_data<Data>(&mut self, data: Data) -> Hash
    where Hasher: DataHasher<Hash, Data>
    {
        self.hasher.hash_data(data)
    }

    /// Add a leaf.
    /// 
    /// If you need push hash you can use [`Self::push`]
    /// 
    /// If you need push many elements, better use
    /// [`Self::push_batched`] & [`Self::push_batched_data`] 
    /// they are faster.
    pub fn push_data<Data>(&mut self, data: Data) -> LeafId
    where Hasher: DataHasher<Hash, Data>
    {
        let hash = self.hash_data(data);
        self.push(hash)
    }
    
    /// Replace a leaf.
    /// 
    /// If you need replace a hash you can use [`Self::replace`]
    /// 
    /// If you need replace many elements in a row, better use 
    /// [`Self::replace_batched_data`] 
    /// they are faster.
    pub fn replace_data<Data>(&mut self, data: Data, id: LeafId)
    where Hasher: DataHasher<Hash, Data>
    {
        let hash = self.hash_data(data);
        self.replace(hash, id)
    }
}
