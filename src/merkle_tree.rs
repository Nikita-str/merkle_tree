use std::ops::Range;
use crate::utility::{get_pad_index, length_in_base};
use crate::MtArityHasher as ArityHasher;
use crate::MtDataHasher as DataHasher;
use crate::MtDataHasherStatic as StaticDataHasher;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] Merkle Tree Level

/// `Mt` stands for `MerkleTree`
/// 
/// Structure that test equality of levels.\
/// # Examples
/// Next levels are equal:
/// ```txt
/// 1A. || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4
/// 1B. || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4 4 | 3 4 4 ||
/// 1C. || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4 4 | 3 4 4 || 0 1 2 | 3 4 4 | 3 4 4 ||
/// 
/// 2A. || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 7 7 ||  
/// 2B. || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 7 7 || 7 
/// because they both will be equal to: 
/// 2(equal). || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 7 7 || 7 7 7 | 7 7 7 | 7 7 7 ||
/// ```
/// And next are **not** equal:
/// ```txt
/// 1A. || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4       !
/// 1B. || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4 4 | 3 3 4 ||
/// because they will be equal to: 
/// 1A(equal). || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4 4 | 3 3 4 ||
/// 1B(equal). || 9 8 7 | 7 8 9 | 9 1 9 || 0 1 2 | 3 4 4 | 3 3 4 ||
/// 
/// 2A. || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 8 7 ||  
/// 2B. || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 8 7 || 7 
/// because they will be equal to: 
/// 2A(equal). || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 8 7 || 7 7 7 | 7 7 7 | 7 8 7 ||
/// 2B(equal). || 1 2 3 | 4 5 6 | 0 0 5 || 7 7 7 | 7 7 7 | 7 8 7 || 7 7 7 | 7 7 7 | 7 7 7 ||
/// ```
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

// [-] Merkle Tree Level
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeafId(usize);
impl LeafId {
    #[inline(always)]
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    #[inline(always)]
    pub fn index(self) -> usize {
        self.0
    }
}

/// You can get NodeId by [MerkleTree::node_id_by_parent_of_leaf]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId {
    pub lvl: usize,
    pub index: usize,
}

pub type MerkleBinTree<Hash, Hasher> = MerkleTree<Hash, Hasher, 2>; 
pub type MerkleTrinaryTree<Hash, Hasher> = MerkleTree<Hash, Hasher, 3>; 

#[derive(Debug, Clone)]
pub struct MerkleTree<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> {
    tree_lvls: Vec<Vec<Hash>>,
    hasher: Box<Hasher>,

    add_lvl_sz: usize,
    new_lvl_cap: usize,
}
// TODO: swap remove
// TODO: diff
// TODO: subtree
// TODO: extend / continuation to lvl & calc root (of n-th lvl)

impl<Hash: Eq, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    pub const ARITY: usize = ARITY;

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
        }
    }
    pub fn new_from_leafs<I>(hasher: Hasher, leafs_iter: I) -> Self
    where I: IntoIterator<Item = Hash>
    {
        let mut tree = Self::new_minimal(hasher);
        tree.push_batched(leafs_iter);
        tree
    }
    pub fn new_from_data<I, Data>(hasher: Hasher, leafs_iter: I) -> Self
    where
        I: IntoIterator<Item = Data>,
        Hasher: StaticDataHasher<Hash, Data>,
    {
        let mut tree = Self::new_minimal(hasher);
        tree.push_batched_data(leafs_iter);
        tree
    }
    
    pub fn node_id_by_parent_of_leaf(&self, leaf: LeafId, lvl: usize) -> NodeId {
        let index = leaf.0 / ARITY.pow(lvl as u32);
        NodeId {
            lvl,
            index
        }
    }
    
    #[inline(always)]
    pub fn is_valid_node_id(&self, node_id: NodeId) -> bool {
        let lvl = node_id.lvl;
        let index = node_id.index;
        self.height() < lvl && self.lvl_len(lvl) < index
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
        if self.is_empty() { return 0 }
        self.tree_lvls.len()
    }
    #[inline]
    fn leaf_amount(&self) -> usize {
        // never panic becasue `tree_lvls[0]` always defined
        self.lvl_len(0)
    }
    #[inline]
    fn next_leaf_id(&self) -> LeafId {
        // never panic becasue `tree_lvls[0]` always defined
        LeafId(self.lvl_len(0))
    }
    /// # painc
    /// * if `lvl >= self.height()`
    #[inline]
    fn lvl_len(&self, lvl: usize) -> usize {
        self.tree_lvls[lvl].len()
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

    pub fn is_valid_leaf_id(&self, id: LeafId) -> bool {
        id.0 < self.leaf_amount()
    }
}
impl<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    fn lvl_must(&self) -> usize {
        length_in_base(self.tree_lvls[0].len() - 1, ARITY) as usize + 1
    }

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
        if self.leaf_amount() == self.add_lvl_sz {
            self.add_lvl_sz *= ARITY;
            self.tree_lvls.push(Vec::with_capacity(self.new_lvl_cap));
        }

        let elem_n = self.leaf_amount();
        self.tree_lvls[0].push(hash);
        self.recalc_elem_hashes(elem_n);

        LeafId::new(self.leaf_amount() - 1)
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
        self.replace_batched(batch, self.next_leaf_id())
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
        if start_id > self.next_leaf_id() {
            panic!("invalid `start_id`")
        }

        let from = start_id;
        let mut to = start_id.0;

        let mut ended = false;
        let mut batch = batch.into_iter();
        
        // replacing hashes from batches while they in valid range  
        for index in start_id.0..self.leaf_amount() {
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
            to = self.leaf_amount();
        }
        let to = LeafId::new(to);

        // update hashes on next levels:
        if from != to {
            let mut from = from.0;
            let mut to = to.0;
            let mut lvl = 1;
            let lvl_must = self.lvl_must();

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
            if other.leaf_amount() == 0 { continue }
            let other_height = other.height();

            let mut recalc_index: Option<usize> = None;
            for (lvl, tree_lvl) in other.tree_lvls.into_iter().enumerate() {
                self.make_lvl_valid(lvl, tree_lvl.len());
                
                if let Some(from_index) = recalc_index {
                    let pre_lvl_range = from_index..self.lvl_len(lvl - 1);
                    self.calc_lvl_hashes(pre_lvl_range, lvl);
                    recalc_index = Some(from_index / ARITY);
                } else {
                    let left_len = self.lvl_len(lvl);
                    self.tree_lvls[lvl].extend(tree_lvl);
                    if left_len % ARITY != 0 {
                        recalc_index = Some(left_len);
                    }
                }
            }

            // now calc top if needed
            let mut recalc_index = recalc_index.unwrap_or(0);
            let lvl_must = self.lvl_must();
            for lvl in other_height..lvl_must {
                let pre_lvl_len = self.lvl_len(lvl - 1);
                self.make_lvl_valid(lvl,  pre_lvl_len / ARITY);
                
                let pre_lvl_range = recalc_index..pre_lvl_len;
                self.calc_lvl_hashes(pre_lvl_range, lvl);
                recalc_index = recalc_index / ARITY;
            }
        }
    }

    /// You can get NodeId by [Self::node_id_by_parent_of_leaf]
    /// 
    /// # Panic
    /// * if `!self.is_valid_node_id`
    #[inline]
    pub fn get_node_ref(&self, node_id: NodeId) -> &Hash {
        &self.tree_lvls[node_id.lvl][node_id.index]
    }
}
impl<Hash: Clone, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// You can get NodeId by [Self::node_id_by_parent_of_leaf]
    /// 
    /// # Panic
    /// * if `!self.is_valid_node_id`
    #[inline]
    pub fn get_node(&self, node_id: NodeId) -> Hash {
        self.get_node_ref(node_id).clone()
    }

    /// Calculate hash of node `node_id` without write it into the node.
    ///
    /// You can get NodeId by [Self::node_id_by_parent_of_leaf]
    /// 
    /// # Panic
    /// * if `!self.is_valid_node_id`
    /// * if `!self.hasher.is_the_same(&hasher)`
    pub fn recalc_node(&self, node_id: NodeId, hasher: &mut Hasher) -> Hash {
        if !self.hasher.is_the_same(&hasher) {
            panic!("hashers is not equal")
        }

        if node_id.lvl == 0 {
            return self.tree_lvls[0][node_id.index].clone()
        }

        let lvl = node_id.lvl - 1;

        let index_start = node_id.index * ARITY;
        let index_end = (index_start + ARITY).min(self.lvl_len(lvl));
        let repeat_last = (index_start + ARITY) - index_end;
        
        for index in index_start..index_end {
            hasher.hash_arity_one_ref(&self.tree_lvls[lvl][index]);
        }
        for _ in 0..repeat_last {
            hasher.hash_arity_one_ref(&self.tree_lvls[lvl][index_end - 1]);
        }

        hasher.finish_arity()
    }
}
impl<Hash: Clone + Eq, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// Verify if node `node_id` have correct hash.
    /// 
    /// You can get NodeId by [Self::node_id_by_parent_of_leaf]
    /// 
    /// # Panic
    /// * if `!self.is_valid_node_id`
    pub fn verify_node(&self, node_id: NodeId, hasher: &mut Hasher) -> bool {
        self.recalc_node(node_id, hasher) == self.tree_lvls[node_id.lvl][node_id.index]
    }
}
impl<Hash: Clone, Hasher: Clone + ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// Last tree can have less height than `lvl`.
    /// 
    /// # painc
    /// * if `lvl >= self.height()`    
    pub fn split(&self, lvl: usize) -> Vec<Self> {
        if lvl == 0 && self.is_empty() {
            return vec![self.clone()]
        }

        let len = self.lvl_len(lvl);
        let hasher =self.hasher.as_ref();
        let mut trees: Vec<Self> = (0..len).into_iter().map(|_|Self::new_minimal(hasher.clone())).collect();
        trees.iter_mut().for_each(|tree|tree.tree_lvls.clear());

        for cur_lvl in 0..=lvl {
            let chunk_size = ARITY.pow((lvl - cur_lvl) as u32);
            for (tree_index, tree_lvl) in self.tree_lvls[cur_lvl].chunks(chunk_size).enumerate() {
                trees[tree_index].tree_lvls.push(tree_lvl.iter().cloned().collect());
            }
        }
        if let Some(tree) = trees.last_mut() {
            let height = tree.lvl_must();
            tree.tree_lvls.truncate(height);
        }
        for tree in &mut trees {
            if tree.tree_lvls.is_empty() {
                tree.tree_lvls.push(vec![])
            }
        }

        trees
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
impl<Hash, Hasher: ArityHasher<Hash, ARITY>, const ARITY: usize> MerkleTree<Hash, Hasher, ARITY> {
    /// # panic 
    /// * if `!self.is_valid_leaf_id(id)`
    pub fn proof_ref(&self, id: LeafId) -> MtProofRef<'_, Hash, ARITY> {
        assert!(self.height() != 0);

        let mut index = id.0;
        let mut lvl = 0;
        let mut tree_lvl_nodes = vec![];
        let mut tree_lvl_path = vec![];

        while lvl + 1 < self.height() {
            let tree_lvl = &self.tree_lvls[lvl];

            let next_index = index / ARITY;
            let index_start = next_index * ARITY;
            let index_end = (index_start + ARITY).min(tree_lvl.len());
            tree_lvl_nodes.push(&tree_lvl[index_start..index_end]);
            tree_lvl_path.push(index % ARITY);
            index = next_index;
            lvl += 1;
        }

        return MtProofRef {
            tree_lvl_nodes,
            tree_lvl_path,
            root: self.root_ref(),
        }
    }

    /// Locally better use [`Self::proof_ref`].
    /// 
    /// It needs in cases when you need to send Proof somewhere.
    /// 
    /// # panic
    /// * if `!self.is_valid_leaf_id(id)`
    pub fn proof_owned(&self, id: LeafId) -> MtProof<Hash, ARITY>
    where Hash: Clone
    {
        self.proof_ref(id).to_owned()
    }

    #[cfg(feature = "serde")]
    pub fn serializable(&self) -> MtSerde<Hash, ARITY>
    where Hash: Clone
    {
        MtSerde::from_merkle_tree(&self)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] MerkleTree Serde

#[cfg(feature = "serde")]
impl<
    Hash: serde::Serialize, 
    Hasher: ArityHasher<Hash, ARITY>, 
    const ARITY: usize
> serde::Serialize for MerkleTree<Hash, Hasher, ARITY> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        MtSerdeRef::from_merkle_tree(self).serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de, 
    Hash: serde::Deserialize<'de> + Eq + Clone + std::fmt::Debug, 
    Hasher: ArityHasher<Hash, ARITY> + Default, 
    const ARITY: usize
> serde::Deserialize<'de> for MerkleTree<Hash, Hasher, ARITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let mt_serde = MtSerde::deserialize(deserializer)?;
        mt_serde.to_merkle_tree(Hasher::default()).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
#[derive(thiserror::Error, Debug)]
pub enum MerkleTreeSerdeError<Hash> {
    #[error("Invalid arity. Expected arity {0}, but it was {1}. Just use correct arity for the tree.")]
    InvalidArity(usize, usize),
    #[error("Wrong root(expected: {0:?}; was: {1:?}). There is potentially an error in the algorithm. Please report this.")]
    WrongRoot(Hash, Hash),
    #[error("Tree must be empty, but it's not?!")]
    ExpectedEmptyTree,
}
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
/// `Mt` stands for `MerkleTree`
pub struct MtSerde<Hash, const ARITY: usize> {
    leafs: Vec<Hash>,
    root: Option<Hash>,
    arity: usize,
}
#[cfg(feature = "serde")]
impl<Hash, const ARITY: usize> MtSerde<Hash, ARITY> {
    pub fn to_merkle_tree<Hasher>(self, hasher: Hasher) -> Result<MerkleTree<Hash, Hasher, ARITY>, MerkleTreeSerdeError<Hash>>
    where
        Hash: Eq + Clone,
        Hasher: ArityHasher<Hash, ARITY>
    {
        if self.arity != ARITY {
            return Err(MerkleTreeSerdeError::InvalidArity(ARITY, self.arity));
        }

        let tree = MerkleTree::new_from_leafs(hasher, self.leafs);
        if let Some(root) = self.root {
            if tree.root_ref() != &root {
                return Err(MerkleTreeSerdeError::WrongRoot(root, tree.root()));
            }
        } else {
            if !tree.is_empty() {
                return Err(MerkleTreeSerdeError::ExpectedEmptyTree);
            }
        }

        Ok(tree)
    }
    pub fn from_merkle_tree<Hasher>(mt: &MerkleTree<Hash, Hasher, ARITY>) -> Self
    where
        Hash: Clone,
        Hasher: ArityHasher<Hash, ARITY>
    {
        let leafs = mt.tree_lvls[0].clone();
        if mt.is_empty() {
            Self {
                leafs,
                root: None,
                arity: ARITY,
            }
        } else {
            Self {
                leafs,
                root: Some(mt.root()),
                arity: ARITY,
            }
        }
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
/// `Mt` stands for `MerkleTree`
pub struct MtSerdeRef<'tree, Hash, const ARITY: usize> {
    leafs: &'tree Vec<Hash>,
    root: Option<&'tree Hash>,
    arity: usize,
}
#[cfg(feature = "serde")]
impl<'tree, Hash, const ARITY: usize> MtSerdeRef<'tree, Hash, ARITY> {
    pub fn from_merkle_tree<Hasher>(mt: &'tree MerkleTree<Hash, Hasher, ARITY>) -> Self
    where Hasher: ArityHasher<Hash, ARITY>
    {
        let leafs = &mt.tree_lvls[0];
        if mt.is_empty() {
            Self {
                leafs,
                root: None,
                arity: ARITY,
            }
        } else {
            Self {
                leafs,
                root: Some(mt.root_ref()),
                arity: ARITY,
            }
        }
    }
}

// [-] MerkleTree Serde
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] MerkleTree Proof

/// `Mt` stands for `MerkleTree`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MtProof<Hash, const ARITY: usize> {
    /// SHOULD have len: `ARITY * LVLs` 
    tree_lvl_nodes: Vec<Hash>,
    tree_lvl_path: Vec<usize>,
    root: Hash,
}
impl<Hash: Eq, const ARITY: usize> MtProof<Hash, ARITY> {
    pub fn verify<Hasher>(&self, mut hash: Hash, hasher: &mut Hasher) -> bool
    where Hasher: ArityHasher<Hash, ARITY>
    {
        for (cur_lvl, path_index) in self.tree_lvl_path.iter().copied().enumerate() {
            let is_valid = self.tree_lvl_nodes[cur_lvl * ARITY + path_index] == hash;
            if !is_valid { return false }

            for index in 0..ARITY {
                hasher.hash_arity_one_ref(&self.tree_lvl_nodes[cur_lvl * ARITY + index]);
            }
            hash = hasher.finish_arity();
        }
        hash == self.root
    }

    pub fn verify_data<Data, Hasher>(&self, data: Data, hasher: &mut Hasher) -> bool
    where Hasher: ArityHasher<Hash, ARITY> + DataHasher<Hash, Data>
    {
        let hash = hasher.hash_data(data);
        self.verify(hash, hasher)
    }
}

/// `Mt` stands for `MerkleTree`
pub struct MtProofRef<'tree, Hash, const ARITY: usize> {
    tree_lvl_nodes: Vec<&'tree [Hash]>,
    tree_lvl_path: Vec<usize>,
    root: &'tree Hash,
}
impl<'tree, Hash, const ARITY: usize> Clone for MtProofRef<'tree, Hash, ARITY> {
    fn clone(&self) -> Self {
        Self { 
            tree_lvl_nodes: self.tree_lvl_nodes.clone(), 
            tree_lvl_path: self.tree_lvl_path.clone(), 
            root: self.root,
        }
    }
}
impl<'tree, Hash: Eq, const ARITY: usize> MtProofRef<'tree, Hash, ARITY> {
    pub fn verify<Hasher>(&self, mut hash: Hash, hasher: &mut Hasher) -> bool
    where Hasher: ArityHasher<Hash, ARITY>
    {
        for (cur_lvl, path_index) in self.tree_lvl_path.iter().copied().enumerate() {
            let is_valid = self.tree_lvl_nodes[cur_lvl][path_index] == hash;
            if !is_valid { return false }

            let nodes_amount = self.tree_lvl_nodes[cur_lvl].len();
            for hash in self.tree_lvl_nodes[cur_lvl] {
                hasher.hash_arity_one_ref(hash);
            }

            // if we have unaligned amount of nodes on current lvl 
            // => we need to hash last node few times more 
            for _ in 0..(ARITY - nodes_amount) {
                hasher.hash_arity_one_ref(&self.tree_lvl_nodes[cur_lvl][nodes_amount - 1]);
            }

            hash = hasher.finish_arity();
        }
        &hash == self.root
    }

    pub fn verify_data<Data, Hasher>(&self, data: Data, hasher: &mut Hasher) -> bool
    where Hasher: ArityHasher<Hash, ARITY> + DataHasher<Hash, Data>
    {
        let hash = hasher.hash_data(data);
        self.verify(hash, hasher)
    }
}
impl<'tree, Hash: Clone, const ARITY: usize> MtProofRef<'tree, Hash, ARITY> {
    pub fn to_owned(self) -> MtProof<Hash, ARITY> {
        let mut tree_lvl_nodes = Vec::with_capacity(self.tree_lvl_nodes.len() * ARITY);
        for lvl_nodes in self.tree_lvl_nodes {
            let len = lvl_nodes.len();

            for index in 0..len {
                tree_lvl_nodes.push(lvl_nodes[index].clone());
            }

            // align it if unaligned:
            for _ in len..ARITY {
                tree_lvl_nodes.push(lvl_nodes[len - 1].clone());
            }
        }

        MtProof {
            tree_lvl_nodes,
            tree_lvl_path: self.tree_lvl_path,
            root: self.root.clone(),
        }
    }
}

// [-] MerkleTree Proof
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
