use std::hash::DefaultHasher as StdHasher;

/// `Mt` stands for `MerkleTree`
pub trait MtHasher<Hash> {
    fn hash_one_ref(&mut self, hash: &Hash);
    fn finish(&mut self) -> Hash;
    
    /// Test if two hashers are equal.
    /// 
    /// **For most cases shuld just return true:**
    /// ```ignore
    /// fn is_the_same(&self, _: &Self) -> bool {
    ///     true
    /// }
    /// ``` 
    /// Needs another impl if Hasher itself contains parameters
    /// that changes result of hash function.
    /// 
    /// # Example
    /// If your hasher have additional data that change resulted hash itself:
    /// ```ignore
    /// struct HasherWithPrefix {
    ///     data_prefix: Vec<u8>,
    ///     hasher_sha256,
    /// } 
    /// ```
    /// You need impl this function as next:
    /// ```ignore
    /// fn is_the_same(&self, other: &Self) -> bool {
    ///     self.data_prefix == other.data_prefix
    /// }
    /// ```
    fn is_the_same(&self, other: &Self) -> bool; 
}

/// `Mt` stands for `MerkleTree`
pub trait MtArityHasher<Hash, const ARITY: usize> {
    fn hash_arity_one_ref(&mut self, hash: &Hash);
    fn finish_arity(&mut self) -> Hash;

    fn hash_arity(&mut self, hashes: &[Hash; ARITY]) -> Hash {
        for hash in hashes {
            self.hash_arity_one_ref(hash);
        }
        self.finish_arity()
    }
    
    /// Test if two hashers are equal.
    /// 
    /// **For most cases shuld just return true:**
    /// ```ignore
    /// fn is_the_same(&self, _: &Self) -> bool {
    ///     true
    /// }
    /// ``` 
    /// Needs another impl if Hasher itself contains parameters
    /// that changes result of hash function.
    /// 
    /// # Example
    /// If your hasher have additional data that change resulted hash itself:
    /// ```ignore
    /// struct HasherWithPrefix {
    ///     data_prefix: Vec<u8>,
    ///     hasher_sha256,
    /// } 
    /// ```
    /// You need impl this function as next:
    /// ```ignore
    /// fn is_the_same(&self, other: &Self) -> bool {
    ///     self.data_prefix == other.data_prefix
    /// }
    /// ```
    fn is_the_same(&self, other: &Self) -> bool; 
}
impl<Hasher: MtHasher<Hash>, Hash, const ARITY: usize> MtArityHasher<Hash, ARITY> for Hasher {
    fn hash_arity_one_ref(&mut self, hash: &Hash) {
        self.hash_one_ref(hash);
    }
    fn finish_arity(&mut self) -> Hash {
        self.finish()
    }
    fn is_the_same(&self, other: &Self) -> bool {
        self.is_the_same(other)
    }
}

/// `Mt` stands for `MerkleTree`
pub trait MtDataHasher<Hash, Data> {
    fn hash_data(&mut self, data: Data) -> Hash;
}

/// `Mt` stands for `MerkleTree`
/// 
/// Static data hasher that doesn't need `&mut self`
pub trait MtDataHasherStatic<Hash, Data>: MtDataHasher<Hash, Data> {
    fn hash_data_static(data: Data) -> Hash;
}
impl<Hasher, Hash, Data> MtDataHasher<Hash, Data> for Hasher
where Hasher: MtDataHasherStatic<Hash, Data>,
{
    fn hash_data(&mut self, data: Data) -> Hash {
        Self::hash_data_static(data)
    }
}

#[cfg(any(feature = "unsecure", test))]
#[derive(Debug, Clone)]
pub struct UnsecureHasher {
    inner: StdHasher,
}

impl Default for UnsecureHasher {
    fn default() -> Self {
        Self::new()
    }
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
    fn is_the_same(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(any(feature = "unsecure", test))]
impl<Data: std::hash::Hash> MtDataHasherStatic<u64, Data> for UnsecureHasher {
    fn hash_data_static(data: Data) -> u64 {
        let mut hasher = Self::new();
        data.hash(&mut hasher.inner);
        hasher.finish()
    }
}
