use sha2::{Sha256, Digest};

use crate::{MtDataHasher, MtHasher};

#[derive(Clone, PartialEq, Eq)]
pub struct Hash {
    hash: [u8; 32]
}
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.hash
    }
}
impl Hash {
    pub fn be_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
    pub fn into_be_bytes(self) -> [u8; 32] {
        self.hash
    }
    pub fn into_iter_be_bytes(self) -> impl Iterator<Item = u8> {
        self.hash.into_iter()
    }
    pub fn iter_be_bytes(&self) -> impl Iterator<Item = &u8> {
        self.hash.iter()
    }
    pub fn into_iter_le_bytes(self) -> impl Iterator<Item = u8> {
        self.hash.into_iter().rev()
    }
    pub fn iter_le_bytes(&self) -> impl Iterator<Item = &u8> {
        self.hash.iter().rev()
    }

    pub fn deserialize_str(hash_str: &str) -> Result<Self, &'static str> {
        if hash_str.len() != 64 { return Err("incorrect hash len") }

        let mut hash = [0u8; 32];
        let mut first_half = true;
        let mut index = 0;
        let mut val = 0;
        for c in hash_str.chars() {
            let half = if ('0'..='9').contains(&c) {
                c as u8 - b'0'
            } else {
                c as u8 - b'a' + 10
            };

            val = (val << 4) + half;
            if !first_half {
                hash[31 - index] = val; // reversed order of bytes (LE)
                index += 1;
                val = 0;
            } 

            first_half = !first_half;
        }

        Ok(Self { hash })
    }
}
impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash_str = self.to_string();
        f.write_str(&hash_str)
    }
}
impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // reversed order of bytes (LE)
        let hash_str = self.iter_le_bytes() 
            .fold(String::new(), |s, x|serialize_u8_as_hex(s, *x));
        f.write_str(&hash_str)
    }
}
fn serialize_u8_as_hex(mut s: String, x: u8) -> String {
    let as_hex = |x|{
        if x < 10 { ('0' as u8 + x) as char }
        else { ('a' as u8 + x - 10) as char }
    };
    s.push(as_hex(x / 16));
    s.push(as_hex(x % 16));
    s
}
#[cfg(feature = "serde")]
impl serde::Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        let hash_str = self.to_string();
        serializer.serialize_str(&hash_str)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        let hash_str = String::deserialize(deserializer)?;
        Hash::deserialize_str(&hash_str).map_err(serde::de::Error::custom)
    }
}

pub struct BitcoinHasher {
    inner: Sha256,
}
impl BitcoinHasher {
    pub fn new() -> Self {
        Self { inner: Sha256::new() }
    }
}
impl Default for BitcoinHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl MtHasher<Hash> for BitcoinHasher {
    fn hash_one_ref(&mut self, hash: &Hash) {

        self.inner.update(hash);
    }
    fn finish(&mut self) -> Hash {
        let hasher = std::mem::take(&mut self.inner);

        let hash = hasher.finalize();
        let mut second_hasher = Sha256::new();
        second_hasher.update(hash);
        let hash = second_hasher.finalize();
        assert_eq!(hash.len(), 32);
        
        let hash = hash[0..32].try_into().unwrap();
        Hash{ hash }
    }
    fn is_the_same(&self, _: &Self) -> bool {
        true
    }
}

impl<Data: AsRef<[u8]>> MtDataHasher<Hash, Data> for BitcoinHasher {
    fn hash_data(&mut self, data: Data) -> Hash {
        self.inner.update(data);
        self.finish()
    }
}

impl<Data: AsRef<[u8]>> crate::MtDataHasherStatic<Hash, Data> for BitcoinHasher {
    fn hash_data_static(data: Data) -> Hash {
        Self::new().hash_data(data)
    }
}
