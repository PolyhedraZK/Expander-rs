use std::fmt;
use std::fmt::{Debug, Display};

use arith::{Field, FieldSerde};
use ark_std::{end_timer, log2, start_timer};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{Leaf, Node, Path};

/// Represents a Merkle tree structure.
#[derive(Clone, Debug, PartialEq)]
pub struct Tree<F: Field + FieldSerde> {
    pub nodes: Vec<Node>,
    pub leaves: Vec<Leaf<F>>,
}

impl<F: Field + FieldSerde> Display for Tree<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "nodes:")?;
        for (i, e) in self.nodes.iter().enumerate() {
            writeln!(f, "{}: {}", i, e)?;
        }
        writeln!(f, "leaves:")?;
        for (i, e) in self.leaves.iter().enumerate() {
            writeln!(f, "{}: {}", i, e)?;
        }
        Ok(())
    }
}

impl<F: Field + FieldSerde> Tree<F> {
    /// Creates an empty tree with default leaves.
    #[inline]
    pub fn init(tree_height: usize) -> Self {
        let leaves = vec![Leaf::default(); 1 << (tree_height - 1)];
        Self::new_with_leaves(leaves)
    }

    /// Builds a tree with the given leaves.
    #[inline]
    pub fn new_with_field_elements(leave_elems: &[F]) -> Self {
        let leaves = leave_elems
            .iter()
            .map(|&leaf| Leaf { data: leaf })
            .collect::<Vec<Leaf<F>>>();
        Self::new_with_leaves(leaves)
    }

    /// Builds a tree with the given leaves.
    #[inline]
    pub fn new_with_leaves(leaves: Vec<Leaf<F>>) -> Self {
        let tree_height = log2(leaves.len() + 1);

        let leaf_nodes = leaves
            .as_slice()
            .iter()
            .map(|leaf| leaf.leaf_hash())
            .collect::<Vec<Node>>();
        let nodes = Self::new_with_leaf_nodes(leaf_nodes, tree_height);
        Self {
            nodes: [nodes.0, nodes.1].concat(),
            leaves,
        }
    }

    /// Builds a tree with pre-hashed leaf nodes.
    ///
    /// # Arguments
    ///
    /// * `leaf_nodes` - Vector of pre-hashed leaf nodes
    /// * `tree_height` - Height of the tree
    ///
    /// # Returns
    ///
    /// A tuple containing vectors of non-leaf nodes and leaf nodes.
    pub fn new_with_leaf_nodes(leaf_nodes: Vec<Node>, tree_height: u32) -> (Vec<Node>, Vec<Node>) {
        let timer = start_timer!(|| format!("generate new tree with {} leaves", leaf_nodes.len()));

        let len = leaf_nodes.len();
        assert_eq!(len, 1 << (tree_height - 1), "incorrect leaf size");

        let mut non_leaf_nodes = vec![Node::default(); (1 << (tree_height - 1)) - 1];

        // Compute the starting indices for each non-leaf level of the tree
        let mut index = 0;
        let mut level_indices = Vec::with_capacity(tree_height as usize - 1);
        for _ in 0..(tree_height - 1) {
            level_indices.push(index);
            index = left_child_index(index);
        }

        // Compute the hash values for the non-leaf bottom layer
        {
            let start_index = level_indices.pop().unwrap();
            let upper_bound = left_child_index(start_index);

            non_leaf_nodes
                .par_iter_mut()
                .enumerate()
                .take(upper_bound)
                .skip(start_index)
                .for_each(|(current_index, e)| {
                    let left_leaf_index = left_child_index(current_index) - upper_bound;
                    let right_leaf_index = right_child_index(current_index) - upper_bound;
                    *e = Node::node_hash(
                        &leaf_nodes[left_leaf_index],
                        &leaf_nodes[right_leaf_index],
                    );
                });
        }

        // Compute the hash values for nodes in every other layer in the tree
        level_indices.reverse();

        for &start_index in &level_indices {
            let upper_bound = left_child_index(start_index);
            let mut buf = non_leaf_nodes[start_index..upper_bound].to_vec();
            buf.par_iter_mut().enumerate().for_each(|(index, node)| {
                *node = Node::node_hash(
                    &non_leaf_nodes[left_child_index(index + start_index)],
                    &non_leaf_nodes[right_child_index(index + start_index)],
                );
            });
            non_leaf_nodes[start_index..upper_bound].clone_from_slice(buf.as_ref());
        }
        end_timer!(timer);

        (non_leaf_nodes, leaf_nodes.to_vec())
    }

    /// Returns the root node of the tree.
    #[inline]
    pub fn root(&self) -> Node {
        self.nodes[0]
    }

    /// Generates a membership proof for the given index.
    #[inline]
    pub fn gen_proof(&self, index: usize, tree_height: usize) -> Path<F> {
        let timer = start_timer!(|| "generate membership proof");

        // Leaf
        let leaf_index_in_tree = convert_index_to_last_level(index, tree_height);
        let leaf = self.leaves[index];

        // Path nodes
        let sibling_index_in_tree = sibling_index(leaf_index_in_tree).unwrap();
        let mut path_nodes = Vec::with_capacity(tree_height - 1);
        path_nodes.push(self.nodes[sibling_index_in_tree]);

        // Iterate from the bottom layer after the leaves to the top
        let mut current_node = parent_index(leaf_index_in_tree).unwrap();
        while current_node != 0 {
            let sibling_node = sibling_index(current_node).unwrap();
            path_nodes.push(self.nodes[sibling_node]);
            current_node = parent_index(current_node).unwrap();
        }

        path_nodes.reverse();
        end_timer!(timer);
        Path {
            index,
            leaf,
            path_nodes,
        }
    }
}

/// Returns the index of the sibling, given an index.
#[inline]
fn sibling_index(index: usize) -> Option<usize> {
    if index == 0 {
        None
    } else if is_left_child(index) {
        Some(index + 1)
    } else {
        Some(index - 1)
    }
}

/// Returns the index of the parent, given an index.
#[inline]
fn parent_index(index: usize) -> Option<usize> {
    if index > 0 {
        Some((index - 1) >> 1)
    } else {
        None
    }
}

/// Returns the index of the left child, given an index.
#[inline]
fn left_child_index(index: usize) -> usize {
    2 * index + 1
}

/// Returns the index of the right child, given an index.
#[inline]
fn right_child_index(index: usize) -> usize {
    2 * index + 2
}

/// Converts a leaf index to its position in the last level of the tree.
#[inline]
fn convert_index_to_last_level(index: usize, tree_height: usize) -> usize {
    index + (1 << (tree_height - 1)) - 1
}

/// Returns true if the given index represents a left child.
#[inline]
fn is_left_child(index: usize) -> bool {
    index % 2 == 1
}
