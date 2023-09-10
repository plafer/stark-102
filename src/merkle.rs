use std::rc::Rc;

use blake3::Hash;

use crate::{field::BaseField, util::is_power_of_2};

/// A Merkle tree implementation that uses blake3 as a hashing function
pub struct MerkleTree {
    pub leaves: Vec<LeafNode>,
    pub root: Hash,
}

impl MerkleTree {
    pub fn new(leaf_values: &[BaseField]) -> Self {
        if !is_power_of_2(leaf_values.len()) {
            panic!("Merkle tree expects leaves to be power of 2")
        }

        // create leaves

        todo!()
    }
}

pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

impl Node {
    /// Only the root node will return `None`
    pub fn parent(&self) -> Option<Rc<Node>> {
        match self {
            Node::Internal(node) =>
                node.parent
                    .as_ref()
                    .cloned(),
            Node::Leaf(node) => 
                node.parent
                    .as_ref()
                    .cloned()
        }
    }

    /// Only leaf nodes will return `None`
    pub fn left(&self) -> Option<Rc<Node>> {
        match self {
            Node::Internal(node) => Some(
                node.left
                    .as_ref()
                    .cloned()
                    .expect("internal node has no left child"),
            ),

            Node::Leaf(_) => None,
        }
    }

    /// Only leaf nodes will return `None`
    pub fn right(&self) -> Option<Rc<Node>> {
        match self {
            Node::Internal(node) => Some(
                node.right
                    .as_ref()
                    .cloned()
                    .expect("internal node has no right child"),
            ),

            Node::Leaf(_) => None,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Node::Internal(node) => node.hash,
            Node::Leaf(node) => node.hash,
        }
    }
}

pub struct LeafNode {
    parent: Option<Rc<Node>>,
    hash: Hash,
}

pub struct InternalNode {
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
    parent: Option<Rc<Node>>,
    hash: Hash,
}
