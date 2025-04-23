use std::str::FromStr;
use rand::{Rng, RngCore};
use crate::{merkle_tree::{LeafId, MerkleTrinaryTree, MtLvl}, utility::length_in_base, MerkleTree, MtDataHasher, NodeId, UnsecureHasher};

fn unsecure_hash(x: u64) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    std::hash::Hasher::write_u64(&mut hasher, x);
    std::hash::Hasher::finish(&hasher)
}
fn unsecure_hash_v(r: &[u64]) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    for x in r {
        std::hash::Hasher::write_u64(&mut hasher, *x);
    }
    std::hash::Hasher::finish(&hasher)
}
fn unsecure_next_lvl_hash(prev_lvl: &[u64], arity: usize) -> Vec<u64> {
    prev_lvl.chunks(arity).map(|x|{
        let last = *x.last().unwrap();

        let mut x = x.to_vec();
        for _ in x.len()..arity {
            x.push(last);
        }

        unsecure_hash_v(&x)
    }).collect()
} 

#[test]
fn tree_height_test() {
    let hasher = UnsecureHasher::new(); 
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), []); 
    assert_eq!(tree.height(), 0);
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), [9]); 
    assert_eq!(tree.height(), 1);
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), [9, 10]);
    assert_eq!(tree.height(), 2);
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), [9, 10, 11]);
    assert_eq!(tree.height(), 2);
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), [9, 10, 11, 12]);
    assert_eq!(tree.height(), 3);
    let tree = MerkleTrinaryTree::new_from_leafs(hasher.clone(), [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(tree.height(), 3);
}

#[test]
fn unsecure_mt_push_test() {
    const RAND_N: usize = 5;
    let mut rng = rand::rng();

    // ARITY = 3
    let mut vecs: Vec<Vec<u64>> = vec![];
    vecs.push((1..=8).collect());
    vecs.push((1..=7).collect());
    for _ in 0..RAND_N {
        vecs.push((1..=8).map(|_|rng.next_u64()).collect());
    }

    for vec in vecs {
        let arity = 3;
        let hasher = UnsecureHasher::new();
        let mut tree = MerkleTree::<u64, UnsecureHasher, 3>::new_minimal(hasher);
        for data in vec.clone() {
            tree.push_data(data);
        }

        let mut hasher = UnsecureHasher::new();
        for lvl in 0..tree.height() {
            let len = tree.get_lvl(lvl).len();
            for index in 0..len {
                let node_id = NodeId { lvl, index };
                let expected = tree.get_node(node_id);
                assert_eq!(expected, tree.recalc_node(node_id, &mut hasher));
            }
        }
        assert_eq!(tree.height(), 3);
    
    
        let mut awaited = vec![];
        for data in vec.clone() {
            awaited.push(unsecure_hash(data))
        }
        assert_eq!(tree.get_lvl(0).to_vec(), &awaited, "init vec is {vec:?}");
    
        let mut lvl = 1; 
        loop {
            awaited = unsecure_next_lvl_hash(&awaited, arity);
            assert_eq!(tree.get_lvl(lvl).to_vec(), &awaited, "init vec is {vec:?}");
            if awaited.len() == 1 { break; }
            lvl += 1;
        }
    }

    // ARITY = 5
    let mut vecs: Vec<Vec<u64>> = vec![];
    vecs.push((1..=25).collect());
    vecs.push((1..=23).collect());
    vecs.push((1..=22).collect());
    vecs.push((1..=123).collect());
    vecs.push((1..=94).collect());
    vecs.push((1..=400).map(|x|x * x + 7).collect());
    for _ in 0..RAND_N {
        vecs.push((1..=(rng.next_u32() as usize % 110) + 5).map(|_|rng.next_u64()).collect());
    }

    for vec in vecs {
        let arity = 5;
        let hasher = UnsecureHasher::new();
        let mut tree = MerkleTree::<u64, UnsecureHasher, 5>::new_minimal(hasher);
        for data in vec.clone() {
            tree.push_data(data);
        }
        
        let mut hasher = UnsecureHasher::new();
        for lvl in 0..tree.height() {
            let len = tree.get_lvl(lvl).len();
            for index in 0..len {
                let node_id = NodeId { lvl, index };
                let expected = tree.get_node(node_id);
                assert_eq!(expected, tree.recalc_node(node_id, &mut hasher));
            }
        }
    
        let mut awaited = vec![];
        for data in vec.clone() {
            awaited.push(unsecure_hash(data))
        }
        assert_eq!(tree.get_lvl(0).to_vec(), &awaited, "init vec is {vec:?}");
    
        let mut lvl = 1; 
        loop {
            awaited = unsecure_next_lvl_hash(&awaited, arity);
            assert_eq!(tree.get_lvl(lvl).to_vec(), &awaited, "init vec is {vec:?}");
            if awaited.len() == 1 { break; }
            lvl += 1;
        }
    }
}

pub fn to_vec<T: FromStr>(s: &str) -> Vec<T>
where <T as FromStr>::Err: std::fmt::Debug
{
    let s = s.replace('|', "");
    let s = s.replace('_', "");
    let s = s.replace("..", "");
    s.split(" ")
        .filter(|x|!x.is_empty())
        .map(|s|str::parse::<T>(s).unwrap())
        .collect::<Vec<_>>()
}

pub fn to_vec_u64(s: &str) -> Vec<u64> {
    to_vec(s)
}

#[test]
fn aux_test_to_vec() {
    assert_eq!(
        to_vec_u64("6 7 7 | 6 7 7 | 6 7 8 ||"), 
        vec![6, 7, 7, 6, 7, 7, 6, 7, 8]
    );
    assert_eq!(
        to_vec_u64("0 1 2 | 3 4 5 | 4 5 5 || 6 7 8 |"), 
        vec![0, 1, 2, 3, 4, 5, 4, 5, 5, 6, 7, 8]
    );
    assert_eq!(to_vec_u64("1 2 _ _ _ _"), vec![1, 2, ]);
    assert_eq!(to_vec_u64("1 2 2 2 2 2"), vec![1, 2, 2, 2, 2, 2]);
    assert_eq!(to_vec_u64("1 2 2 2 2 2 | _ _ _ _ _ _ | .."), vec![1, 2, 2, 2, 2, 2]);
}


#[test]
fn unsecure_mt_lvl_eq_test() {
    fn test<const ARITY: usize>(a: &Vec<u64>, b: &Vec<u64>, eq: bool) {
        let a = MtLvl::<_, ARITY>::new(&a);
        let b = MtLvl::<_, ARITY>::new(&b);
        if eq {
            assert_eq!(a, b);
            assert_eq!(b, a);
        } else {
            assert_ne!(a, b);
            assert_ne!(b, a);
        }
    }

    let a = to_vec_u64("1 2 3 | 4");
    let b = to_vec_u64("1 2 3 | 4 4 4");
    test::<3>(&a, &b, true);

    let a = to_vec_u64("1 2 3 4 5 | 7 8");
    let b = to_vec_u64("1 2 3 4 5 | 7 8 8 8 8 | 7 8 8 8 8 | 7 8 8");
    test::<5>(&a, &b, true);

    let a = to_vec_u64("1 2 3 4 5 | 7 8");
    let b = to_vec_u64("1 2 3 4 5 | 7 8 8 8 8 | 7 8");
    test::<5>(&a, &b, true);

    
    let a = to_vec_u64("1 2 _ _ _ _");
    let b = to_vec_u64("1 2 2 2 2 2");
    test::<6>(&a, &b, true);
    
    let a = to_vec_u64("1 2 3 | 4 5 5 | _ _ _");
    let b = to_vec_u64("1 2 3 | 4 5 5 | 4 _ _");
    test::<3>(&a, &b, false);
    
    let a = to_vec_u64("6 7 7 | 6 7 7 | 6 7 8 || 6 7 7 | 6 7 7 | 6 7 8 || ");
    let b = to_vec_u64("6 7 7 | 6 7 7 | 6 7 8 || 6 7 7 | 6 7 7 | 6 7 8 || 6 7 7 | ");
    test::<3>(&a, &b, false);
    let a = to_vec_u64("6 7 7 | 6 7 7 | 6 7 8 || 6 7 7 | 6 7 7 | 6 7 7 || ");
    test::<3>(&a, &b, false);
    let b = to_vec_u64("6 7 7 | 6 7 7 | 6 7 8 || 6 7 7 | 6 7 7 | 6 7 7 || 6 7 7 | ");
    test::<3>(&a, &b, true);

    let a = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 7 8 || ");
    let b = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 7 8 || 7");
    test::<3>(&a, &b, false);
    let a = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 7 7 || ");
    let b = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 7 7 || 7");
    test::<3>(&a, &b, true);
    let a = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 8 7 || ");
    let b = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 7 | 7 8 7 || 7");
    test::<3>(&a, &b, false);
    let a = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 8 | 7 7 7 || ");
    let b = to_vec_u64("1 2 3 | 4 5 5 | 3 2 1 || 7 7 7 | 7 7 8 | 7 7 7 || 7");
    test::<3>(&a, &b, false);
}

#[test]
fn unsecure_mt_eq_test() {
    fn get_initiated_tree<const ARITY: usize>(vec: &[u64]) -> MerkleTree::<u64, UnsecureHasher, ARITY> {
        let hasher = UnsecureHasher::new();
        let mut tree = MerkleTree::<u64, UnsecureHasher, ARITY>::new_minimal(hasher);
        for data in vec {
            tree.push_data(*data);
        }
        tree
    }
    // all next trees(ARITY is 3) are equal:
    // 1 2 3 | 4 5 _ | _ _ _
    // 1 2 3 | 4 5 5 | _ _ _
    // 1 2 3 | 4 5 5 | 4 5 _
    // 1 2 3 | 4 5 5 | 4 5 5
    // no one of them are equal with:
    // 1 2 3 | 4 5 5 | 4 4 5
    // 1 2 3 | 4 5 5 | 4 5 6
    // 1 2 3 | 4 5 6 | _ _ _
    // 1 2 3 | 4 5 5 | 4 5 5 || 1 _ _ | ... // because of height
    // 1 2 3 | 4 5 5 | 4 _ _ // because it is `1 2 3 | 4 5 5 | 4 4 4`
    let non_eq = [
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 6]),
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 4, 5]),
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 5, 6]),
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 5, 6]),
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4]),
        get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 5, 5, 1]),
    ];
    let a = get_initiated_tree::<3>(&[1, 2, 3, 4, 5]);

    let assert = |b| {
        assert!(a.eq_full(&b), "a = b test");
        assert!(b.eq_full(&a), "b = a test");
        for lvl in 0..b.height() {
            assert_eq!(a.get_lvl(lvl), b.get_lvl(lvl), "lvl {lvl} test");
        }
        non_eq.iter().for_each(|x|{
            assert!(x.ne_full(&b), "x != b test");
            assert!(&b.ne_full(&x), "b != x test");
            assert_ne!(b.get_lvl(0), x.get_lvl(0), "non eq lvl 0 test");
        });
    };

    let b = get_initiated_tree::<3>(&[1, 2, 3, 4, 5]);
    assert(b);

    let b = get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5]);
    assert(b);
    
    let b = get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 5]);
    assert(b);
    
    let b = get_initiated_tree::<3>(&[1, 2, 3, 4, 5, 5, 4, 5, 5]);
    assert(b);
    
    // next trees are equal:
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 _ | ... 
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8 7 6 | 9 3
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8 7 6 | 9 3 3 | 9 3 3
    // and non eq with:
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8 7 6 | 
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8 7 6 | 9 
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8 7 6 | 9 3 3 | 9<9>3
    // 1 2 3 | 4 5 1 | 4 2 5 || 8 7 6 | 9 3 3 | 9 3 3 || 8<6>6 | 9 3 3 | 9 3 3
    let a = [1, 2, 3, 4, 5, 1, 4, 2, 5, /* || */ 8, 7, 6, 9, 3];
    let a = get_initiated_tree::<3>(&a);

    let b = [
        1, 2, 3, 4, 5, 1, 4, 2, 5, 
        8, 7, 6, 9, 3, 3, 9, 3, 3, 
        8, 7, 6, 9, 3, 3, 9, 3, 3,
    ];
    let b = get_initiated_tree::<3>(&b);
    assert!(a.eq_full(&b));
    assert!(b.eq_full(&a));

    let not_eq: &[&[_]] = &[
        &[
            1, 2, 3, 4, 5, 1, 4, 2, 5, 
            8, 7, 6, 9, 3, 3, 9, 3, 3, 
            8, 7, 6,
        ],
        &[
            1, 2, 3, 4, 5, 1, 4, 2, 5, 
            8, 7, 6, 9, 3, 3, 9, 3, 3, 
            8, 7, 6, 9,
        ],
        &[
            1, 2, 3, 4, 5, 1, 4, 2, 5, 
            8, 7, 6, 9, 3, 3, 9, 9, 3, 
            8, 7, 6, 9, 3, 3, 9, 3, 3,
        ],
        &[
            1, 2, 3, 4, 5, 1, 4, 2, 5, 
            8, 7, 6, 9, 3, 3, 9, 3, 3, 
            8, 6, 6, 9, 3, 3, 9, 3, 3,
        ],
    ];
    let not_eq: Vec<_> = not_eq.iter().map(|x|get_initiated_tree::<3>(x)).collect();
    for ne in &not_eq {
        assert!(a.ne_full(ne));
        assert!(b.ne_full(ne));
        assert!(ne.ne_full(&b));
        assert!(ne.ne_full(&a));
    }
    not_eq.iter().for_each(|not_eq|{
        assert_ne!(a.get_lvl(0), not_eq.get_lvl(0), "non eq lvl 0 test");
        assert_ne!(b.get_lvl(0), not_eq.get_lvl(0), "non eq lvl 0 test");
    });
    
    let b = [
        1, 2, 3, 4, 5, 1, 4, 2, 5, 
        8, 7, 6, 9, 3, 3, 9, 3, 3, 
        8, 7, 6, 9, 3, 3
    ];
    let b = get_initiated_tree::<3>(&b);
    assert!(a.eq_full(&b));
    assert!(b.eq_full(&a));
    for ne in &not_eq {
        assert!(b.ne_full(ne));
        assert!(ne.ne_full(&b));
    }
    not_eq.iter().for_each(|not_eq|{
        assert_ne!(a.get_lvl(0), not_eq.get_lvl(0), "non eq lvl 0 test");
        assert_ne!(b.get_lvl(0), not_eq.get_lvl(0), "non eq lvl 0 test");
    });
}

#[cfg(test)]
#[test]
fn vec_continuation_test() {
    let a = to_vec_u64("7 7 5| 5 5 7 | 9 8 7 || 1 2 3 | 4 5 _ | _ _ _");
    let b = to_vec_u64("7 7 5| 5 5 7 | 9 8 7 || 1 2 3 | 4 5 5 | 4 5 5 || 1 2 3 | 4 5 5 | 4 5 5");
    assert_eq!(MtLvl::<_, 3>::vec_continuation(a), b);
    
    let a = to_vec_u64("1 2");
    let b = to_vec_u64("1 2 2 2 2 2");
    assert_eq!(MtLvl::<_, 6>::vec_continuation(a), b);
    
    let a = to_vec_u64("1 2 3 4 5 6 | 9 8 7");
    let mut b = to_vec_u64("1 2 3 4 5 6");
    b.extend(to_vec_u64("9 8 7 7 7 7").repeat(5));
    assert_eq!(MtLvl::<_, 6>::vec_continuation(a), b);
}

#[cfg(test)]
#[test]
fn add_hasher_test() {
    use crate::{merkle_tree::LeafId, MtDataHasher};

    let mut vecs: Vec<Vec<u64>> = vec![];
    vecs.push((1..=25).collect());
    vecs.push((1..=23).collect());
    vecs.push((1..=22).collect());
    vecs.push((1..=123).collect());
    vecs.push((1..=94).collect());
    for vec in vecs {
        let arity = 5;
        let mut tree = MerkleTree::<u64, AddHasher, 5>::new_minimal(AddHasher::default());
        for data in vec.clone() {
            tree.push_data(data);
        }
    
        let mut hasher = AddHasher::default(); 
        let mut awaited = vec![];
        for data in vec.clone() {
            awaited.push(hasher.hash_data(data))
        }
        assert_eq!(tree.get_lvl(0).to_vec(), &awaited, "init vec is {vec:?}");
    
        let mut lvl = 1; 
        loop {
            awaited = AddHasher::next_lvl_hash(awaited.as_slice(), arity);
            assert_eq!(tree.get_lvl(lvl).to_vec(), &awaited, "init vec is {vec:?}");
            if awaited.len() == 1 { break; }
            lvl += 1;
        }
    }

    let mut tree = MerkleTree::<u64, AddHasher, 3>::new_minimal(AddHasher::default());
    for i in 0..9 {
        tree.push(i);
        println!("{tree:?}\n");
    }

    tree.replace(101, LeafId::new(1));
    println!("{tree:?}\n");

    
    let mut tree = MerkleTree::<u64, AddHasher, 3>::new_minimal(AddHasher::default());
    tree.push_batched([0, 101, 2, 3, 4, 5, 6, 7, 8]);
    println!("{tree:?}\n");
    
    
    let mut tree = MerkleTree::<u64, AddHasher, 3>::new_minimal(AddHasher::default());
    tree.push_batched([0, 101, 2, 3, 4,]);
    // tree.push_batched([5, 6, 7, 8]);
    println!("{tree:?}\n");
}

#[test]
pub fn push_batched_test() {
    type Hasher = UnsecureHasher; // AddHasher;

    let mut tree2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
    let mut tree3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
    let mut tree5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());

    let mut vec = Vec::with_capacity(100);
    for x in 0..37u64 {
        let mut tree_x2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
        let mut tree_x3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
        let mut tree_x5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
        
        vec.push(tree_x2.hash_data(x));
        tree_x2.push_batched(vec.clone());
        tree_x3.push_batched(vec.clone());
        tree_x5.push_batched(vec.clone());

        tree2.push_data(x);
        tree3.push_data(x);
        tree5.push_data(x);

        assert!(tree2.eq_full(&tree_x2), "x = {x}\n{tree2:?}\n=?=\n{tree_x2:?}");
        assert!(tree3.eq_full(&tree_x3), "x = {x}");
        assert!(tree5.eq_full(&tree_x5), "x = {x}");
    }

    let vecss = vec![
        vec![(1u64..9).collect::<Vec<_>>(), (9..=25).collect(), (26..35).collect()],
        vec![(1..7).collect::<Vec<_>>(), (9..=25).collect(), (26..35).collect()],
        vec![(1..17).collect::<Vec<_>>(), (17..37).collect(), (37..59).collect()],
        vec![(1..16).collect::<Vec<_>>(), (16..38).collect(), (38..60).collect()],
        vec![(1..17).collect::<Vec<_>>(), (100..134).collect(), (200..229).collect()],
        vec![(1..15).collect::<Vec<_>>(), (25..34).collect(), (72..76).collect(), (2..3).collect(), (205..235).collect()],
    ];

    for vecs in vecss {
        let mut tree2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
        let mut tree3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
        let mut tree5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
    
        let mut tree_x2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
        let mut tree_x3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
        let mut tree_x5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());

        for vec in vecs {
            let vec: Vec<_> = vec.into_iter().map(|data|tree2.hash_data(data)).collect();

            for x in vec.clone() {
                tree2.push(x);
                tree3.push(x);
                tree5.push(x);
            }
            tree_x2.push_batched(vec.clone());
            tree_x3.push_batched(vec.clone());
            tree_x5.push_batched(vec.clone());
            
            assert!(tree2.eq_full(&tree_x2));
            assert!(tree3.eq_full(&tree_x3));
            assert!(tree5.eq_full(&tree_x5));
        }
    }
}

#[test]
pub fn replace_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();

    let mut vecs = vec![
        (1u64..=9).collect::<Vec<_>>(),
        (1u64..=24).collect::<Vec<_>>(),
        (1u64..=39).collect::<Vec<_>>(),
        (1u64..=25).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=27).map(|_|rng.next_u64()).collect::<Vec<_>>(),
    ];

    for _repeats in 0..13 {
        for vec in vecs.iter_mut() {
            let mut tree2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
            let mut tree3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
            let mut tree5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
            tree2.push_batched(vec.clone());
            tree3.push_batched(vec.clone());
            tree5.push_batched(vec.clone());
            
            let index = rng.random_range(0..vec.len());
            let new_hash = rng.next_u64(); 
            vec[index] = new_hash;
            tree2.replace(new_hash, LeafId::new(index));
            tree3.replace(new_hash, LeafId::new(index));
            tree5.replace(new_hash, LeafId::new(index));
            
            let mut tree2x = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
            let mut tree3x = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
            let mut tree5x = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
            tree2x.push_batched(vec.clone());
            tree3x.push_batched(vec.clone());
            tree5x.push_batched(vec.clone());
            
            assert!(tree2.eq_full(&tree2x));
            assert!(tree3.eq_full(&tree3x));
            assert!(tree5.eq_full(&tree5x));
        }
    }
}

#[test]
pub fn replace_batched_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();

    let test = |vec_init: Vec<_>, vec_result: Vec<_>, vec_replaces: Vec<(Vec<_>, _)>| {
        let mut tree2 = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
        let mut tree3 = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
        let mut tree5 = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
        let mut tree5d = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
        tree2.push_batched(vec_init.clone());
        tree3.push_batched(vec_init.clone());
        tree5.push_batched(vec_init.clone());
        tree5d.push_batched_data(vec_init.clone());
    
        for (vec_repl, start_id) in vec_replaces {
            tree2.replace_batched(vec_repl.clone(), start_id);
            tree3.replace_batched(vec_repl.clone(), start_id);
            tree5.replace_batched(vec_repl.clone(), start_id);
            tree5d.replace_batched_data(vec_repl.clone(), start_id);
        }
        
        let mut tree2x = MerkleTree::<u64, Hasher, 2>::new_minimal(Hasher::new());
        let mut tree3x = MerkleTree::<u64, Hasher, 3>::new_minimal(Hasher::new());
        let mut tree5x = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
        let mut tree5xd = MerkleTree::<u64, Hasher, 5>::new_minimal(Hasher::new());
        tree2x.push_batched(vec_result.clone());
        tree3x.push_batched(vec_result.clone());
        tree5x.push_batched(vec_result.clone());
        tree5xd.push_batched_data(vec_result.clone());
    
        assert!(tree3.eq_full(&tree3x));
        assert!(tree2.eq_full(&tree2x));
        assert!(tree5.eq_full(&tree5x));
        assert!(tree5d.eq_full(&tree5xd));
    };

    let vec_in_ = to_vec_u64("0 1 2 | 3 4 5 | 6  7  8 ||  9 10 11 | 12 13 14 | 15 16 _ ||");
    let vec_out = to_vec_u64("0 1 2 | 3 4 5 | 6 37 38 || 39 30 31 | 32 33 14 | 15 16 _ ||");
    let replaces = vec![(vec![37, 38, 39, 30, 31, 32, 33], LeafId::new(7))];
    test(vec_in_, vec_out, replaces);
    
    let vec_in_ = to_vec_u64("0 1 2 ");
    let vec_out = to_vec_u64("0 1 2 | 3 4 5 | 6");
    let replaces = vec![(vec![3, 4, 5, 6], LeafId::new(3))];
    test(vec_in_, vec_out, replaces);
    
    let vec_in_ = to_vec_u64("0 1 2 | 3 4 5 | 6 7 8 ||  9 10 11 | 12 13 14 |  15  16   _ ||");
    let vec_out = to_vec_u64("0 1 2 | 3 4 5 | 6 7 8 || 39 30 31 | 32 33 14 | 105 106 107 || 108 109 _ |");
    let replaces = vec![
        (vec![39, 30, 31, 32, 33], LeafId::new(9)),
        (vec![105, 106, 107, 108, 109], LeafId::new(15)),
    ];
    test(vec_in_, vec_out, replaces);
    let vec_in_ = to_vec_u64("0 1  2 |  3  4  5 |  6  7  8 ||  9 10 11 | 12 13 14 | 15 16 _ ||");
    let vec_out = to_vec_u64("0 1 52 | 53 54 55 | 56 37 38 || 39 40 41 | 42 43 14 | 15 16 _ ||");
    let replaces = vec![
        (vec![37, 38, 39, 30, 31, 32, 33], LeafId::new(7)),
        (vec![40, 41, 42, 43], LeafId::new(10)),
        (vec![], LeafId::new(13)),
        (vec![], LeafId::new(17)),
        (vec![52, 53, 54, 55, 56], LeafId::new(2)),
        (vec![], LeafId::new(11)),
    ];
    test(vec_in_, vec_out, replaces);

    // random test:
    for _repeats in 0..20 {
        let vec_in: Vec<_> = (0..rng.random_range(12..=123)).map(|_|rng.next_u64()).collect();
        let mut vec_out = vec_in.clone();
        let mut replaces = vec![];
        for _ in 0..rng.random_range(1..=7) {
            let v: Vec<_> = (0..rng.random_range(0..=17)).map(|_|rng.next_u64()).collect();
            let id = rng.random_range(0..=vec_out.len());

            let mut v_index = 0;
            for index in id..vec_out.len() {
                if v_index < v.len() {
                    vec_out[index] = v[v_index];
                    v_index += 1;
                } else {
                    break;
                }
            }
            vec_out.extend_from_slice(&v[v_index..]);

            replaces.push((v, LeafId::new(id)));
        }

        test(vec_in, vec_out, replaces);    
    }
}


#[test]
pub fn merge_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();

    let test_2 = |a: Vec<_>, b: Vec<_>| {
        let a_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), a.clone());
        let b_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), b.clone());
    
        let merged = MerkleTree::new_merged([a_tree, b_tree]).unwrap();
        
        let mut ab = a.clone();
        ab.extend(b.clone());
        let expected = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), ab);
    
        assert!(expected.eq_full(&merged), "{expected:?}\n=?=\n{merged:?}\n");
    };

    let a_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), vec![]);
    let b_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), vec![]);
    let merged = MerkleTree::new_merged([a_tree.clone(), b_tree.clone()]).unwrap();
    assert!(merged.is_empty());
    let c = to_vec_u64("0 1 2 | 4 ..");
    let c_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), c);
    let merged = MerkleTree::new_merged([a_tree.clone(), c_tree.clone(), b_tree.clone()]).unwrap();
    assert!(c_tree.eq_full(&merged));

    let a = to_vec_u64("0 1 2 | ..");
    let b = to_vec_u64("3 4 5 | 6 7 8 | 9 10 11");
    test_2(a.clone(), b.clone());
    test_2(b, a);

    let a = to_vec_u64("0 1 2 | 4 5 ..");
    let b = to_vec_u64("6 7 8 | 9 10 11 | 12 13 _");
    test_2(a.clone(), b.clone());
    test_2(b, a);
    
    let a = to_vec_u64("0 1 2 | 3 4 5 | 6");
    let b = to_vec_u64(" 7 8 || 9 10 11 | 12 13 14 | 15 16 17 ||");
    test_2(a.clone(), b.clone());
    test_2(b, a);

    let a = to_vec_u64("0 1 2 | 3 4 5 | 6 7 8 || 99 ");
    test_2(a.clone(), a.clone());
    let a_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), a.clone());
    let weird_tree = a_tree.split(2).get(1).unwrap().clone();
    let b = to_vec_u64(" 7 8 || 9 10 11 | 12 13 14 | 15 16 17 ||");
    let b_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), b.clone());
    let tree = MerkleTree::new_merged([weird_tree.clone(), b_tree.clone()]).unwrap();
    let expected = to_vec_u64(" 99 7 8 || 9 10 11 | 12 13 14 | 15 16 17 ||");
    let expected = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), expected);
    assert!(tree.eq_full(&expected));
    let tree = MerkleTree::new_merged([b_tree.clone(), weird_tree.clone()]).unwrap();
    let expected = to_vec_u64(" 7 8 || 9 10 11 | 12 13 14 | 15 16 17 || 99 ");
    let expected = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), expected);
    assert!(tree.eq_full(&expected));
    let tree = MerkleTree::new_merged([weird_tree.clone(), weird_tree.clone()]).unwrap();
    let expected = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), [99, 99]);
    assert!(tree.eq_full(&expected));

    let mut vecss = vec![
        vec![(1..=9).collect::<Vec<u64>>(), (10..=19).collect(), (20..=29).collect()],
        vec![(1..=9).collect(), (1 + 10..=27 + 10).collect(), (1 + 100..=27*2 + 100).collect()],
        vec![(1 + 10..=27 + 10).collect(), (1..=9).collect(), (1 + 100..=27*2 + 100).collect(), (200..=213).collect()],
    ];
    // generate random test:
    for _repeats in 0..20 {
        let mut v = vec![];
        for _ in 0..rng.random_range(2..=7) {
            let x: Vec<_> = (0..rng.random_range(1..=45)).map(|_|rng.next_u64()).collect();
            v.push(x);
        }
        vecss.push(v);
    }

    for vecs in vecss {
        let trees2 = vecs.clone().into_iter().map(
            |x|MerkleTree::<_, _, 2>::new_from_leafs(Hasher::new(), x)
        );
        let trees3 = vecs.clone().into_iter().map(
            |x|MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), x)
        );
        let trees5 = vecs.clone().into_iter().map(
            |x|MerkleTree::<_, _, 5>::new_from_leafs(Hasher::new(), x)
        );
        let merged2 = MerkleTree::new_merged(trees2).unwrap();
        let merged3 = MerkleTree::new_merged(trees3).unwrap();
        let merged5 = MerkleTree::new_merged(trees5).unwrap();
        
        let mut expected_leafs = vec![];
        for vec in vecs {
            expected_leafs.extend(vec);
        }
        let exp = expected_leafs;
        let expected2 = MerkleTree::<_, _, 2>::new_from_leafs(Hasher::new(), exp.clone());
        let expected3 = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), exp.clone());
        let expected5 = MerkleTree::<_, _, 5>::new_from_leafs(Hasher::new(), exp.clone());
    
        assert!(expected2.eq_full(&merged2), "{expected2:?}\n=?=\n{merged2:?}\n");
        assert!(expected3.eq_full(&merged3), "{expected3:?}\n=?=\n{merged3:?}\n");
        assert!(expected5.eq_full(&merged5), "{expected5:?}\n=?=\n{merged5:?}\n");
    }
}

#[test]
fn split_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();

    fn test<const ARITY: usize>(vec: &Vec<u64>) {
        let x_tree = MerkleTree::<_, _, ARITY>::new_from_leafs(Hasher::new(), vec.clone());
        let lvls = length_in_base(vec.len() - 1, ARITY) as usize;
        for lvl in 0..=lvls {
            let trees = x_tree.clone().split(lvl);
    
            let chunk_sz = ARITY.pow(lvl as u32);
            assert_eq!(trees.len(), vec.chunks(chunk_sz).len());
            for (index, expected) in vec.chunks(chunk_sz).enumerate() {
                let expected = expected.iter().cloned();
                let expected_tree = MerkleTree::<_, _, ARITY>::new_from_leafs(Hasher::new(), expected);
                assert!(trees[index].eq_full(&expected_tree), "{lvl}:\n{:?}\n=?=\n{:?}", trees[index], expected_tree);
            }
        }    
    }

    let a = to_vec_u64("0 1 2 | 3 4 5 | 6 7 8 || 9 10 11");
    test::<2>(&a);
    test::<3>(&a);
    test::<5>(&a);

    let vecs = vec![
        (1u64..=9).collect::<Vec<_>>(),
        (1u64..=24).collect::<Vec<_>>(),
        (1u64..=39).collect::<Vec<_>>(),
        (1u64..=25).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=27).map(|_|rng.next_u64()).collect::<Vec<_>>(),
    ];
    for vec in vecs {
        test::<2>(&vec);
        test::<3>(&vec);
        test::<5>(&vec);        
    }
    
    let a = to_vec_u64("0 1");
    test::<2>(&a);
    test::<3>(&a);
    test::<5>(&a);
    
    let a = to_vec_u64("0");
    test::<2>(&a);
    test::<3>(&a);
    test::<5>(&a);

    let a = to_vec_u64("");
    let x_tree = MerkleTree::<_, _, 3>::new_from_leafs(Hasher::new(), a.clone());
    let mut trees = x_tree.split(0);
    assert!(trees.len() == 1);
    let tree = trees.pop().unwrap();
    assert!(x_tree.eq_full(&tree));
}

#[test]
fn proof_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();

    fn test<const ARITY: usize>(vec: &Vec<u64>) {
        let x_tree = MerkleTree::<_, _, ARITY>::new_from_data(Hasher::new(), vec.clone());
        let mut hasher = Hasher::new();
        let hasher = &mut hasher;

        for (id, data) in vec.iter().copied().enumerate() {
            let proof = x_tree.proof_ref(LeafId::new(id));
            let proof_owned = x_tree.proof_owned(LeafId::new(id));
            let hash = hasher.hash_data(data);

            assert!(!proof.verify_data(data + 3, hasher));
            assert!(proof.verify(hash, hasher));
            assert!(proof.verify_data(data, hasher));
            assert!(proof_owned.verify(hash, hasher));
            assert!(proof_owned.verify_data(data, hasher));

            for (_, data2) in vec.iter().copied().enumerate() {
                if data == data2 { continue; }

                let hash2 = hasher.hash_data(data2);
                assert!(!proof.verify(hash2, hasher));
                assert!(!proof.verify_data(data2, hasher));
                assert!(!proof_owned.verify(hash2, hasher));
                assert!(!proof_owned.verify_data(data2, hasher));
             }
        }
    }
    
    let a = to_vec_u64("0 1");
    test::<2>(&a);
    test::<3>(&a);
    test::<5>(&a);
    
    let a = to_vec_u64("0");
    test::<2>(&a);
    test::<3>(&a);
    test::<5>(&a);

    let vecs = vec![
        (1u64..=4).collect::<Vec<_>>(),
        (1u64..=9).collect::<Vec<_>>(),
        (1u64..=12).collect::<Vec<_>>(),
        (1u64..=24).collect::<Vec<_>>(),
        (1u64..=39).collect::<Vec<_>>(),
        (1u64..=25).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=27).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=58).map(|_|rng.next_u64()).collect::<Vec<_>>(),
    ];
    for vec in vecs {
        test::<2>(&vec);
        test::<3>(&vec);
        test::<5>(&vec);        
    }
}

#[cfg(feature = "serde_json")]
#[test]
fn serde_test() {
    type Hasher = UnsecureHasher; // AddHasher;
    let mut rng = rand::rng();
    
    fn test<const ARITY: usize>(vec: &Vec<u64>) {
        let x_tree = MerkleTree::<_, _, ARITY>::new_from_data(Hasher::new(), vec.clone());
        let tree_s = serde_json::to_string(&x_tree).unwrap();
        let tree_deser = serde_json::from_str(&tree_s).unwrap();
        assert!(x_tree.eq_full(&tree_deser));

        // for cases when we there no Default impl for Hasher
        let tree_deser: crate::MtSerde<_, ARITY> = serde_json::from_str(&tree_s).unwrap();
        let tree_deser = tree_deser.to_merkle_tree(Hasher::new()).unwrap();
        assert!(x_tree.eq_full(&tree_deser));

        if vec.len() > 8 {
            let proof = x_tree.proof_owned(LeafId::new(7));
            let proof_ser = serde_json::to_string(&proof).unwrap();
            let proof_deser: crate::MtProof<u64, ARITY> = serde_json::from_str(&proof_ser).unwrap();
            
            let mut hasher = Hasher::new();
            assert!(proof.verify_data(vec[7], &mut hasher));
            assert!(!proof.verify_data(vec[8], &mut hasher));
            assert!(!proof.verify_data(vec[5], &mut hasher));
            
            assert!(proof_deser.verify_data(vec[7], &mut hasher));
            assert!(!proof_deser.verify_data(vec[8], &mut hasher));
            assert!(!proof_deser.verify_data(vec[5], &mut hasher));
        }
    }

    let vecs = vec![
        (1u64..=4).collect::<Vec<_>>(),
        vec![],
        vec![1],
        vec![1, 2],
        (1u64..=9).collect::<Vec<_>>(),
        (1u64..=12).collect::<Vec<_>>(),
        (1u64..=24).collect::<Vec<_>>(),
        (1u64..=39).collect::<Vec<_>>(),
        (1u64..=25).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=27).map(|_|rng.next_u64()).collect::<Vec<_>>(),
        (1u64..=58).map(|_|rng.next_u64()).collect::<Vec<_>>(),
    ];
    for vec in &vecs {
        test::<2>(vec);
        test::<3>(vec);
        test::<5>(vec);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] AddHasher

/// ⛔ It is TOTALY INCORRECT HASH that used only for tests
#[derive(Debug, Default, Clone)]
struct AddHasher {
    acc: u64,
}
impl AddHasher {  
    #[allow(unused)]
    fn new() -> Self {
        Self::default()
    }    
    fn next_lvl_hash(prev_lvl: &[u64], arity: usize) -> Vec<u64> {
        prev_lvl.chunks(arity).map(|x|{
            let last = *x.last().unwrap();

            let mut x = x.to_vec();
            for _ in x.len()..arity {
                x.push(last);
            }

            Self::default().hash_data(x.as_slice())
        }).collect()
    }
}
#[cfg(any(feature = "unsecure", test))]
impl crate::MtHasher<u64> for AddHasher {
    fn hash_one_ref(&mut self, hash: &u64) {
        self.acc = u64::wrapping_add(self.acc, *hash);
    }
    fn finish(&mut self) -> u64 {
        let ret = self.acc;
        self.acc = 0;
        ret
    }
    fn is_the_same(&self, _: &Self) -> bool {
        true
    }
}
impl crate::MtDataHasher<u64, u64> for AddHasher {
    fn hash_data(&mut self, data: u64) -> u64 {
        data
    }
}
impl crate::MtDataHasher<u64, &[u64]> for AddHasher {
    fn hash_data(&mut self, data: &[u64]) -> u64 {
        let mut ret = 0;
        for x in data {
            ret += *x;
        }
        ret
    }
}

// [-] AddHasher
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
