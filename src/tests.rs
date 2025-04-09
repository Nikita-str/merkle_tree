use rand::RngCore;

use crate::{MerkleTree, UnsecureHasher};

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
fn mt_with_unsecure_hasher_test() {
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
        assert_eq!(tree.get_lvl(0), &awaited, "init vec is {vec:?}");
    
        let mut lvl = 1; 
        loop {
            awaited = unsecure_next_lvl_hash(&awaited, arity);
            assert_eq!(tree.get_lvl(lvl), &awaited, "init vec is {vec:?}");
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
        assert_eq!(tree.get_lvl(0), &awaited, "init vec is {vec:?}");
    
        let mut lvl = 1; 
        loop {
            awaited = unsecure_next_lvl_hash(&awaited, arity);
            assert_eq!(tree.get_lvl(lvl), &awaited, "init vec is {vec:?}");
            if awaited.len() == 1 { break; }
            lvl += 1;
        }
    }
}
