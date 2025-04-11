use std::str::FromStr;
use rand::RngCore;
use crate::{merkle_tree::MtLvl, MerkleTree, UnsecureHasher};

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
        assert_eq!(a, b, "a = b test");
        assert_eq!(b, a, "b = a test");
        for lvl in 0..b.height() {
            assert_eq!(a.get_lvl(lvl), b.get_lvl(lvl), "lvl {lvl} test");
        }
        non_eq.iter().for_each(|x|{
            assert_ne!(x, &b, "x != b test");
            assert_ne!(&b, x, "b != x test");
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
    assert_eq!(a, b);
    assert_eq!(b, a);

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
        assert_ne!(&a, ne);
        assert_ne!(&b, ne);
        assert_ne!(ne, &b);
        assert_ne!(ne, &a);
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
    assert_eq!(a, b);
    assert_eq!(b, a);
    for ne in &not_eq {
        assert_ne!(&b, ne);
        assert_ne!(ne, &b);
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