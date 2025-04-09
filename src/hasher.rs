use std::hash::DefaultHasher as StdHasher;

/// `Mt` stands for `MerkleTree`
pub trait MtHasher<Hash> {
    fn hash_one_ref(&mut self, hash: &Hash);
    fn finish(&mut self) -> Hash;
}

/// `Mt` stands for `MerkleTree`
pub trait MtArityHasher<Hash, const ARITY: usize> {
    fn hash_arity_one_ref(&mut self, hash: &Hash);
    fn finish_arity(&mut self) -> Hash;
}
impl<Hasher: MtHasher<Hash>, Hash, const ARITY: usize> MtArityHasher<Hash, ARITY> for Hasher {
    fn hash_arity_one_ref(&mut self, hash: &Hash) {
        self.hash_one_ref(hash);
    }
    fn finish_arity(&mut self) -> Hash {
        self.finish()
    }
}

/// `Mt` stands for `MerkleTree`
pub trait MtDataHasher<Hash, Data> {
    fn hash_data(&mut self, data: Data) -> Hash;
}


#[cfg(any(feature = "unsecure", test))]
#[derive(Debug)]
pub struct UnsecureHasher {
    inner: StdHasher,
}

#[cfg(any(feature = "unsecure", test))]
impl UnsecureHasher {
    pub fn new() -> Self {
        Self { inner: StdHasher::new() }
    }
}

#[cfg(any(feature = "unsecure", test))]
impl MtHasher<u64> for UnsecureHasher {
    fn hash_one_ref(&mut self, hash: &u64) {
        std::hash::Hasher::write_u64(&mut self.inner, *hash);
    }
    fn finish(&mut self) -> u64 {
        let ret = std::hash::Hasher::finish(&self.inner);
        self.inner = StdHasher::new();
        ret
    }
}

#[cfg(any(feature = "unsecure", test))]
impl<Data: std::hash::Hash> MtDataHasher<u64, Data> for UnsecureHasher {
    fn hash_data(&mut self, data: Data) -> u64 {
        data.hash(&mut self.inner);
        self.finish()
    }
}