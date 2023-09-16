use blake3::{hash, Hash, Hasher};

use crate::field::BaseField;

/// A Channel implements the Fiat-Shamir heuristic.
pub struct Channel {
    current_hash: Hash,
}

impl Channel {
    pub fn new(salt: &[u8]) -> Self {
        Self {
            current_hash: hash(salt),
        }
    }

    pub fn commit(&mut self, commitment: Hash) {
        let mut hasher = Hasher::new();
        hasher.update(self.current_hash.as_bytes());
        hasher.update(commitment.as_bytes());

        self.current_hash = hasher.finalize();
    }

    pub fn random_element(&mut self) -> BaseField {
        let hash_first_4_bytes: [u8; 4] = self.current_hash.as_bytes()[0..4].try_into().unwrap();
        let ret_element: BaseField = i32::from_le_bytes(hash_first_4_bytes).into();

        // this is an arbitrary way to change the current hash, so that we can
        // call `random_element()` multiple times and always get a different one
        self.commit(self.current_hash);

        ret_element
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Get a few random elements and make sure they're different
    #[test]
    pub fn test_random_element() {
        let mut channel = Channel::new(&[42u8]);

        let r1 = channel.random_element();
        let r2 = channel.random_element();
        let r3 = channel.random_element();

        assert_ne!(r1, r2);
        assert_ne!(r2, r3);
    }
}