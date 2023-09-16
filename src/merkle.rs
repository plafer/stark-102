use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, bail, Result};
use blake3::Hash;

use crate::{field::BaseField, util::is_power_of_2};

pub type MerkleRoot = blake3::Hash;

/// Identifies whether a hash corresponds to the left or right sibling.
/// This is necessary in order to properly verify an inclusion proof
#[derive(Debug, PartialEq, Eq)]
pub enum SiblingPosition {
    Left,
    Right,
}

pub struct MerklePath {
    /// Hashes starting from the leaf to right below the root (<hash>, Left)
    /// means that our sibling has hash <hash>, and is the left child of our
    /// parent (such that we are the right child)
    pub path: Vec<(Hash, SiblingPosition)>,
}

impl MerklePath {
    pub fn new(merkle_tree: &MerkleTree, index: usize) -> Result<Self> {
        if index >= merkle_tree.leaves.len() {
            bail!(
                "index {index} out of bounds ({} leaves)",
                merkle_tree.leaves.len()
            );
        }

        let mut path = Vec::new();
        let mut node_runner = merkle_tree.leaves[index].clone();

        while node_runner.borrow().parent().is_some() {
            let current_node = Rc::clone(&node_runner);
            let (maybe_current_node_sibling, sibling_position) = current_node
                .borrow()
                .sibling()
                .ok_or(anyhow!("sibling doesn't exist"))?;

            let current_node_sibling = maybe_current_node_sibling.unwrap();

            path.push((current_node_sibling.borrow().hash(), sibling_position));

            node_runner = current_node.borrow().parent().unwrap();
        }

        Ok(Self { path })
    }
}

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

    pub fn verify_inclusion(&self, element: BaseField, path: MerklePath) -> bool {
        let mut current_hash = blake3::hash(&[element.as_byte()]);

        for (sibling_hash, sibling_position) in path.path {
            current_hash = match sibling_position {
                SiblingPosition::Left => {
                    // sibling hash comes first
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(sibling_hash.as_bytes());
                    hasher.update(current_hash.as_bytes());
                    hasher.finalize()
                }
                SiblingPosition::Right => {
                    // sibling hash comes second
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(current_hash.as_bytes());
                    hasher.update(sibling_hash.as_bytes());
                    hasher.finalize()
                }
            }
        }

        self.root == current_hash
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Internal(InternalNode),
    Leaf(LeafNode),
}

impl Node {
    /// Only the root node will return `None`
    pub fn parent(&self) -> Option<Rc<RefCell<Node>>> {
        match self {
            Node::Internal(node) => node.parent.as_ref().cloned(),
            Node::Leaf(node) => node.parent.as_ref().cloned(),
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

    /// Returns the sibling, as well as whether that sibling is the left or
    /// right child of the parent.
    pub fn sibling(&self) -> Option<(Option<Rc<RefCell<Node>>>, SiblingPosition)> {
        let parent = self.parent()?;

        let sibling_position = {
            let left_child_of_parent = parent.borrow().left().unwrap();
            let left_child_of_parent: &Node = &left_child_of_parent.borrow();

            if left_child_of_parent == self {
                // If I'm to the left, then my sibling is to the right
                SiblingPosition::Right
            } else {
                SiblingPosition::Left
            }
        };

        if sibling_position == SiblingPosition::Left {
            Some((parent.borrow().left(), SiblingPosition::Left))
        } else {
            Some((parent.borrow().right(), SiblingPosition::Right))
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

#[derive(Debug, Eq)]
pub struct LeafNode {
    parent: Option<Rc<RefCell<Node>>>,
    hash: Hash,
}

impl PartialEq for LeafNode {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Debug, Eq)]
pub struct InternalNode {
    // Note: We need the `RefCell` only when constructing to set the pointers
    // right. Once the node is created, we'll never need to mutate.
    // Is there any better way to use it? Perhaps a few lines of unsafe code?
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
    parent: Option<Rc<RefCell<Node>>>,
    hash: Hash,
}

impl PartialEq for InternalNode {
    fn eq(&self, other: &Self) -> bool {
        // Note: we only check the hash, because we have reference cycles, which
        // cause the default implementation of `PartialEq` to stack overflow!
        self.hash == other.hash
    }
}

#[cfg(test)]
mod tests {
    use blake3::hash;

    use super::*;

    #[test]
    pub fn test_tree_structure() {
        let leaves: [BaseField; 4] = [1.into(), 2.into(), 3.into(), 4.into()];

        let tree = MerkleTree::new(&leaves);

        for leaf in tree.leaves {
            let leaf = leaf.borrow();
            assert!(leaf.right().is_none());
            assert!(leaf.left().is_none());

            let parent = leaf.parent().unwrap();
            let parent = parent.borrow();

            assert!(parent.right().is_some());
            assert!(parent.left().is_some());

            let root = parent.parent().unwrap();
            let root = root.borrow();

            assert!(root.right().is_some());
            assert!(root.left().is_some());
            assert!(root.parent().is_none());
        }
    }

    #[test]
    pub fn test_sibling() {
        let left = BaseField::from(1);
        let right = BaseField::from(2);

        let leaves = vec![left, right];

        let tree = MerkleTree::new(&leaves);

        let left_leaf_in_tree = tree.leaves[1].borrow().sibling().unwrap().0.unwrap();
        let left_leaf_in_tree = left_leaf_in_tree.borrow();

        let right_leaf_in_tree = tree.leaves[0].borrow().sibling().unwrap().0.unwrap();
        let right_leaf_in_tree = right_leaf_in_tree.borrow();

        assert_eq!(left_leaf_in_tree.hash(), hash(&vec![left.as_byte()]));
        assert_eq!(right_leaf_in_tree.hash(), hash(&vec![right.as_byte()]));
    }

    #[test]
    pub fn test_proof() {
        let leaves: [BaseField; 4] = [1.into(), 2.into(), 3.into(), 4.into()];

        let tree = MerkleTree::new(&leaves);

        let merkle_path = MerklePath::new(&tree, 3).unwrap();

        assert!(tree.verify_inclusion(4.into(), merkle_path));
    }
}
