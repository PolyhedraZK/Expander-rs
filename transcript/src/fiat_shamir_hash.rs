use std::fmt::Debug;

pub mod sha2_256;
pub use sha2_256::*;

pub mod keccak_256;
pub use keccak_256::*;

pub trait FiatShamirHash: Debug + Clone {
    /// The size of the hash output in bytes.
    const DIGEST_SIZE: usize;

    /// Create a new hash instance.
    fn new() -> Self;

    /// Hash the input into the output.
    fn hash(output: &mut [u8], input: &[u8]);

    /// Hash the input in place.
    fn hash_inplace(buffer: &mut [u8]);
}
