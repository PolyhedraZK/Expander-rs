use std::fmt;
use std::fmt::{Debug, Display};

use arith::{Field, FieldSerde};
use sha2::{Digest, Sha512};

use crate::Node;

/// Represents a leaf in the Merkle tree, containing 64 bytes of data stored in a BabyBearx16.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Leaf<F: Field + FieldSerde> {
    pub data: F,
}

impl<F: Field + FieldSerde> Display for Leaf<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Display the first and last byte of the leaf data for brevity
        write!(f, "leaf: 0x{:02x?}", self.data)
    }
}

impl<F: Field + FieldSerde> Leaf<F> {
    /// Creates a new Leaf with the given data.
    pub fn new(data: F) -> Self {
        Self { data }
    }

    pub fn leaf_hash(&self) -> Node {
        let mut hasher = Sha512::new();
        self.data.serialize_into(&mut hasher).unwrap();

        let result: [u8; 32] = hasher.finalize()[..32].try_into().unwrap();

        Node { data: result }
    }

    // /// Computes the hash of the leaf using Poseidon hash function.
    // ///
    // /// # Arguments
    // ///
    // /// * `hash_param` - The Poseidon hash parameters
    // ///
    // /// # Returns
    // ///
    // /// A Node containing the hash of the leaf data.
    // pub fn leaf_hash(&self, hash_param: &PoseidonBabyBearParams) -> Node {
    //     // Use Poseidon hash for leaf nodes
    //     // Note: This could be replaced with SHA2 if performance requires
    //     let mut state = PoseidonBabyBearState { state: self.data };
    //     hash_param.permute(&mut state);
    //     Node {
    //         data: unsafe {
    //             transmute::<BabyBearx16, [u8; 64]>(state.state)[..32]
    //                 .try_into()
    //                 .unwrap()
    //         },
    //     }
    // }
}

impl<F: Field + FieldSerde> From<F> for Leaf<F> {
    /// Implements the From trait to allow creation of a Leaf from BabyBearx16 data.
    fn from(data: F) -> Self {
        Self { data }
    }
}
