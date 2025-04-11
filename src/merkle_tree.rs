use std::marker::PhantomData;
use crate::MtArityHasher as ArityHasher;
use crate::MtDataHasher as DataHasher;

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
            // in this case tree have different height
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

                        // println!("{} =?= {}", l_index, r_index);
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
                    
                    // println!("{}({}) =?= {}({})", a_pos_start + i, l_index, b_pos_start + i, r_index);
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
        if a.height() != b.height() { return false }

        for lvl in (0..a.height()).rev() {
            if a.get_lvl(lvl) != b.get_lvl(lvl) { return false; }
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

/// returns length of number(`n`) in `base`:
/// # Example
/// * `13` = `121_base3` len is `3`  
/// * `13` = `23_base5` len is `2`
/// * `13` = `D_base16` len is `1`
/// 
/// len of `0` always is `0`
fn length_in_base(mut n: usize, base: usize) -> u32 {
    let mut len = 0;
    while n > 0 {
        n /= base;
        len += 1;
    }
    len
}

/// return valid index for current continuation of `MarkleTree` level:
/// # Example
/// ```txt
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | ...                          <-- valid indexes
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 |  9 10 11 |  9 10 11 || ...   <-- result
/// 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...   <-- index
/// ```
/// 
/// return index is decided by last arity-digit that less in `index` than in `max_valid_index`:
/// ```txt
/// 121202
///     ^  --> 111102
/// 111111
/// ```
/// <br/>
/// 
/// ```txt
/// 120202
///   ^    --> 110202
/// 111111
/// ```
fn get_pad_index(mut index: usize, mut max_valid_index: usize, arity: usize) -> usize {
    if index <= max_valid_index { return index }

    let mut shift = 1;
    
    let mut tail = 0;
    let mut cur_tail = 0;
    
    let mut head = 0;

    while index > 0 {
        let tail_digit = index % arity;
        let head_digit = max_valid_index % arity;

        cur_tail += tail_digit * shift;
        head += head_digit * shift;

        if tail_digit < head_digit {
            tail = cur_tail;
            head = 0;
        }

        shift *= arity;
        index /= arity;
        max_valid_index /= arity;
    }

    head + tail
}

#[cfg(test)]
#[test]
fn length_in_base_test() {
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

    assert_eq!(length_in_base(13, 3), 3);
    assert_eq!(length_in_base(13, 5), 2);
    assert_eq!(length_in_base(13, 10), 2);
    assert_eq!(length_in_base(13, 16), 1);
}

#[cfg(test)]
#[test]
fn get_pad_index_test() {
    // 0 1 2 3 4 | 5 6 7 ...                        <-- (max_valid_index = 7)
    // 0 1 2 3 4 | 5 6 7 7 7 |  5  6  7  7  7 | ... <-- result
    // 0 1 2 3 4 | 5 6 7 8 9 | 10 11 12 13 14 | ... <-- index
    assert_eq!(get_pad_index(14, 7, 5), 7);
    assert_eq!(get_pad_index(13, 7, 5), 7);
    assert_eq!(get_pad_index(7, 14, 5), 7);

    assert_eq!(get_pad_index(9, 7, 5), 7);
    assert_eq!(get_pad_index(10, 7, 5), 5);
    assert_eq!(get_pad_index(11, 7, 5), 6);
    assert_eq!(get_pad_index(7, 11, 5), 7);

    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | ...      <-- (max_valid_index = 11)
    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 |  9 10 11 |  9 10 11 || ...      <-- result
    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...      <-- index
    for j in 0..3 {
        for i in 0..3 {
            assert_eq!(get_pad_index(12 + i + j * 3, 11, 3), 9 + i);
        }
    }
    assert_eq!(get_pad_index(26, 11, 3), 11);
    assert_eq!(get_pad_index(18, 11, 3), 9);
    assert_eq!(get_pad_index(19, 11, 3), 10);
    assert_eq!(get_pad_index(20, 11, 3), 11);
    assert_eq!(get_pad_index(21, 11, 3), 9);
    assert_eq!(get_pad_index(22, 11, 3), 10);
    assert_eq!(get_pad_index(23, 11, 3), 11);

    for k in 1..=2 {
        for i in 0..=11 {
            assert_eq!(get_pad_index(27*k + i, 11, 3), i);
        }
        for j in 0..2 {
            for i in 0..3 {
                assert_eq!(get_pad_index(27*k + 12 + i + j * 3, 11, 3), 9 + i);
            }
        }
    }

    // ... | 24 25 26 ||| 27 (27 + 1) (27 + 2) | (27 + 3) (27 + 4) (27 + 5) ...  <-- (max_valid_index = 27 + 6 - 1)
    // eq
    //                ||| 27 28 29 | 30 31 32 | 30 31 32 || 27 28 29 | 30 31 32 | 30 31 32 || ...
    //                ||| 54 55 56 | 57 58 59 | 60 61 62 || 63 64 65 | 66 67 68 | 69 70 71 || ...
    assert_eq!(get_pad_index(27 * 2 + 6 - 1, 27 + 6 - 1, 3), 27 + 6 - 1);
    for i in 0..6 {
        assert_eq!(get_pad_index(27 * 2 + i, 27 + 6 - 1, 3), 27 + i);
        assert_eq!(get_pad_index(27 * 2 + 9 + i, 27 + 6 - 1, 3), 27 + i);
    }
    for i in 0..3 {
        assert_eq!(get_pad_index(27 * 2 + 6 + i, 27 + 6 - 1, 3), 27 + 3 + i);
        assert_eq!(get_pad_index(27 * 2 + 9 + 6 + i, 27 + 6 - 1, 3), 27 + 3 + i);
    }

    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 __ | ..                <-- valid indexes
    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 13 | 12 13 13 || ...   <-- result
    // 0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || ...   <-- index
    let result = "0 1 2 | 3 4 5 | 6 7 8 || 9 10 11 | 12 13 13 | 12 13 13 || 9 10 11 | 12 13 13 | 12 13 13";
    let result = crate::tests::to_vec(result);
    let indexes = (0..27).into_iter().collect::<Vec<_>>();
    assert_eq!(indexes.len(), result.len());
    for index in indexes {
        assert_eq!(get_pad_index(index, 13, 3), result[index], "index is {index}");
    }
    
    let a: Vec<usize> = (0..=364).collect();
    let result = MtLvl::<_, 3>::vec_continuation(a.clone());
    for k in 0..3 {
        for index in 0..result.len() {
            assert_eq!(get_pad_index(index + k * result.len(), a.len() - 1, 3), result[index]);
        }
    }
    
    let a: Vec<usize> = (0..=363).collect();
    let result = MtLvl::<_, 5>::vec_continuation(a.clone());
    for k in 0..3 {
        for index in 0..result.len() {
            assert_eq!(get_pad_index(index + k * result.len(), a.len() - 1, 5), result[index]);
        }
    }
}
