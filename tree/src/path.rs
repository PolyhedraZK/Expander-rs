use std::fmt;
use std::fmt::{Debug, Display};

use arith::{Field, FieldSerde};
use ark_std::{end_timer, start_timer};

use crate::{Leaf, Node};

/// Represents a path in the Merkle tree, used for proving membership.
#[derive(Clone, Debug, PartialEq)]
pub struct Path<F: Field + FieldSerde> {
    pub(crate) leaf: Leaf<F>,
    pub(crate) path_nodes: Vec<Node>,
    pub(crate) index: usize,
}

impl<F: Field + FieldSerde> Display for Path<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "leaf index: {}", self.index)?;

        let position_list = self.position_list();
        for ((i, node), is_right_node) in self.path_nodes.iter().enumerate().zip(position_list) {
            writeln!(
                f,
                "{}-th node, is right node {}, sibling: {}",
                i, is_right_node, node
            )?;
        }

        Ok(())
    }
}

impl<F: Field + FieldSerde> Path<F> {
    /// Computes the position of on-path nodes in the Merkle tree.
    ///
    /// This function converts the leaf index to a boolean array in big-endian form,
    /// where `true` indicates a right child and `false` indicates a left child.
    #[inline]
    fn position_list(&'_ self) -> impl '_ + Iterator<Item = bool> {
        (0..self.path_nodes.len() + 1).map(move |i| ((self.index >> i) & 1) != 0)
    }

    /// Verifies the path against a given root and leaf.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node of the Merkle tree
    /// * `leaf` - The leaf node to verify
    /// * `hasher` - The Poseidon hash parameters
    ///
    /// # Returns
    ///
    /// `true` if the path is valid, `false` otherwise.
    #[inline]
    pub fn verify(&self, root: &Node) -> bool {
        let timer = start_timer!(|| "path verify");

        let position_list = self.position_list().collect::<Vec<_>>();
        // let leaf_node = self.leaf.leaf_hash(hasher);
        let leaf_node = self.leaf.leaf_hash();
        let mut current_node = leaf_node;

        // Traverse the path from leaf to root
        for (i, node) in self.path_nodes.iter().rev().enumerate() {
            if position_list[i] {
                current_node = Node::node_hash(node, &current_node)
            } else {
                current_node = Node::node_hash(&current_node, node)
            };
        }

        end_timer!(timer);
        if current_node != *root {
            println!("path does not match the root");
            false
        } else {
            true
        }
    }

    /// Return the leaf of the path
    #[inline]
    pub fn leaf(&self) -> &Leaf<F> {
        &self.leaf
    }
}
