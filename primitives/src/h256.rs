use crypto::{fast_hash, slow_hash};
use bytes::{LittleEndian, ByteOrder};
use serde;
use std;

/// H256 length in bytes.
pub const H256_LENGTH: usize = 32;

/// A 256-bit hash.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct H256(pub [u8; H256_LENGTH]);

impl H256 {
    pub fn new() -> H256 {
        H256::default()
    }

    pub fn fast_hash<T: AsRef<[u8]>>(input: T) -> H256 {
        H256(fast_hash(input.as_ref()))
    }

    pub fn slow_hash<T: AsRef<[u8]>>(input: T) -> H256 {
        H256(slow_hash(input.as_ref()))
    }

    fn tree_hash_cnt(count: usize) -> usize {
        assert!(count >= 3);
        assert!(count <= 0x10000000);

        let mut pow = 2;
        while pow < count {
            pow <<= 1;
        }
        pow >> 1
    }

    pub fn tree_hash<T>(hashes: T) -> H256 where T: AsRef<[H256]> {
        let hashes = hashes.as_ref();
        assert!(hashes.len() > 0);

        match hashes.len() {
            0 => panic!("tree hash needs at least one element"),
            1 => hashes[0].clone(),
            2 => {
                let mut buf = [0u8; H256_LENGTH * 2];
                (&mut buf[..H256_LENGTH]).copy_from_slice(hashes[0].as_bytes());
                (&mut buf[H256_LENGTH..]).copy_from_slice(hashes[1].as_bytes());

                H256(fast_hash(&buf))
            },
            count => {
                let cnt = Self::tree_hash_cnt(count);
                let mut ints = Vec::with_capacity(cnt);
                for _ in 0..cnt {
                    ints.push(H256::new());
                }

                for i in 0..(2 * cnt - count) {
                    ints[i] = hashes[i].clone();
                }

                let mut i = 2 * cnt - count;
                let mut j = 2 * cnt - count;

                while j < cnt {
                    let mut buf = [0u8; H256_LENGTH * 2];
                    (&mut buf[..H256_LENGTH]).copy_from_slice(hashes[i].as_bytes());
                    (&mut buf[H256_LENGTH..]).copy_from_slice(hashes[i + 1].as_bytes());

                    ints[j] = H256(fast_hash(&buf));
                    i += 2;
                    j += 1;
                }

                assert!(i == count);

                let mut cnt = cnt;
                while cnt > 2 {
                    cnt >>= 1;

                    let mut i = 0;
                    let mut j = 0;
                    while j < cnt {
                        let mut buf = [0u8; H256_LENGTH * 2];
                        (&mut buf[..H256_LENGTH]).copy_from_slice(ints[i].as_bytes());
                        (&mut buf[H256_LENGTH..]).copy_from_slice(ints[i + 1].as_bytes());

                        ints[j] = H256(fast_hash(&buf));
                        i += 2;
                        j += 1;
                    }
                }

                let mut buf = [0u8; H256_LENGTH * 2];
                (&mut buf[..H256_LENGTH]).copy_from_slice(ints[0].as_bytes());
                (&mut buf[H256_LENGTH..]).copy_from_slice(ints[1].as_bytes());

                H256(fast_hash(&buf))
            }
        }
    }

    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> H256 {
        let bytes = bytes.as_ref();
        assert!(bytes.len() == H256_LENGTH, "invalid hash length");

        let mut h = Self::new();
        h.0.clone_from_slice(bytes);
        h
    }

    pub fn u64_components(&self) -> (u64, u64, u64, u64) {
        let v1 = LittleEndian::read_u64(&self.0[..8]);
        let v2 = LittleEndian::read_u64(&(&self.0[8..])[..8]);
        let v3 = LittleEndian::read_u64(&(&self.0[15..])[..8]);
        let v4 = LittleEndian::read_u64(&self.0[23..]);

        (v1, v2, v3, v4)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for H256 {
    fn from(v: [u8; 32]) -> H256 {
        H256(v)
    }
}

impl<'de> serde::Deserialize<'de> for H256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        impl<'de> serde::de::Visitor<'de> for H256 {
            type Value = H256;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a {} bytes slice", H256_LENGTH)
            }

            fn visit_bytes<E>(mut self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error {
                if v.len() != H256_LENGTH {
                    Err(E::custom(format!("slice length isn't {} bytes", H256_LENGTH)))
                } else {
                    self.0.copy_from_slice(v);
                    Ok(self)
                }
            }
        }
        deserializer.deserialize_bytes(H256::default())
    }
}

impl serde::Serialize for H256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}