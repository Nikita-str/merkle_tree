use std::str::FromStr;
use rand::{Rng, RngCore};
use crate::{merkle_tree::{LeafId, MtLvl}, MerkleTree, MtDataHasher, UnsecureHasher};

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

        assert!(tree2.eq_full(&tree_x2));
        assert!(tree3.eq_full(&tree_x3));
        assert!(tree5.eq_full(&tree_x5));
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] AddHasher

/// ⛔ It is TOTALY INCORRECT HASH that used only for tests
#[derive(Debug, Default)]
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
        self.acc += *hash;
    }
    fn finish(&mut self) -> u64 {
        let ret = self.acc;
        self.acc = 0;
        ret
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
