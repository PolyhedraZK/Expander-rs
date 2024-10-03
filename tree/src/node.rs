use std::fmt;
use std::fmt::Display;

use sha2::{Digest, Sha512};

/// A node in the Merkle tree, representing 32 bytes of data.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Node {
    pub(crate) data: [u8; 32],
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Display the first and last byte of the node for brevity
        write!(f, "node: 0x{:02x?}...{:02x?}", self.data[0], self.data[31])
    }
}

impl Node {
    /// Creates a new Node with the given data.
    pub fn new(data: [u8; 32]) -> Self {
        Self { data }
    }

    /// Computes the hash of two child nodes to create a parent node.
    ///
    /// This function uses SHA-512 for hashing and takes the first 32 bytes of the result.
    ///
    /// # Arguments
    ///
    /// * `left` - The left child node
    /// * `right` - The right child node
    ///
    /// # Returns
    ///
    /// A new Node containing the hash of the two input nodes.
    #[inline]
    pub fn node_hash(left: &Node, right: &Node) -> Node {
        let mut hasher = Sha512::new();
        hasher.update(left.data);
        hasher.update(right.data);
        let result = hasher.finalize();
        Node {
            data: result[..32].try_into().unwrap(),
        }
    }

    /// Returns the data of the node as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
