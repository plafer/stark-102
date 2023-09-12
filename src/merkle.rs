use std::{
    cell::RefCell,
    rc::Rc,
};

use blake3::Hash;

use crate::{field::BaseField, util::is_power_of_2};

/// A Merkle tree implementation that uses blake3 as a hashing function
pub struct MerkleTree {
    pub leaves: Vec<Rc<RefCell<Node>>>,
    pub root: Hash,
}

impl MerkleTree {
    pub fn new(leaf_values: &[BaseField]) -> Self {
        if !is_power_of_2(leaf_values.len()) {
            panic!("Merkle tree expects leaves to be power of 2")
        }

        let leaves: Vec<Rc<RefCell<Node>>> = leaf_values
            .iter()
            .map(|ele| {
                let leaf_hash = {
                    let leaf_bytes: [u8; 1] = [ele.as_byte()];
                    blake3::hash(&leaf_bytes)
                };

                Rc::new(RefCell::new(Node::Leaf(LeafNode {
                    parent: None,
                    hash: leaf_hash,
                })))
            })
            .collect();

        let mut current_layer: Vec<Rc<RefCell<Node>>> = leaves.to_vec();

        while current_layer.len() > 1 {
            current_layer = current_layer
                .as_chunks_mut::<2>()
                .0
                .iter_mut()
                .map(|[left, right]| {
                    let hash = {
                        let mut hasher = blake3::Hasher::new();
                        hasher.update(left.borrow().hash().as_bytes());
                        hasher.update(right.borrow().hash().as_bytes());
                        hasher.finalize()
                    };

                    let internal_node = Rc::new(RefCell::new(Node::Internal(InternalNode {
                        left: Some(left.clone()),
                        right: Some(right.clone()),
                        parent: None,
                        hash,
                    })));

                    left.borrow_mut().set_parent(internal_node.clone());
                    right.borrow_mut().set_parent(internal_node.clone());

                    internal_node
                })
                // FIXME: Find a better way than to collect() on every iteration
                .collect();
        }

        let root_node = current_layer[0].borrow();

        Self {
            leaves,
            root: root_node.hash(),
        }
    }
}

pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

impl Node {
    /// Only the root node will return `None`
    pub fn parent(&self) -> Option<Rc<RefCell<Node>>> {
        match self {
            Node::Internal(node) => node.parent.as_ref().cloned(),
            Node::Leaf(node) => node.parent.as_ref().cloned()
        }
    }

    /// Only leaf nodes will return `None`
    ///
    /// Note: The internal use of `RefCell` bleeds into the API
    pub fn left(&self) -> Option<Rc<RefCell<Node>>> {
        match self {
            Node::Internal(node) => {
                assert!(node.left.is_some());

                node.left.as_ref().cloned()
            }

            Node::Leaf(_) => None,
        }
    }

    /// Only leaf nodes will return `None`
    pub fn right(&self) -> Option<Rc<RefCell<Node>>> {
        match self {
            Node::Internal(node) => {
                assert!(node.right.is_some());

                node.right.as_ref().cloned()
            }
            Node::Leaf(_) => None,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Node::Internal(node) => node.hash,
            Node::Leaf(node) => node.hash,
        }
    }

    pub fn set_parent(&mut self, parent: Rc<RefCell<Node>>) {
        match self {
            Node::Internal(node) => node.parent = Some(parent),
            Node::Leaf(node) => node.parent = Some(parent),
        }
    }
}

pub struct LeafNode {
    parent: Option<Rc<RefCell<Node>>>,
    hash: Hash,
}

pub struct InternalNode {
    // Note: We need the `RefCell` only when constructing to set the pointers
    // right. Once the node is created, we'll never need to mutate.
    // Is there any better way to use it? Perhaps a few lines of unsafe code?
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
    parent: Option<Rc<RefCell<Node>>>,
    hash: Hash,
}
