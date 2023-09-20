use blake3::{hash, Hash, Hasher};

use crate::field::BaseField;

/// A Channel implements the Fiat-Shamir heuristic.
pub struct Channel {
    current_hash: Hash,
    count: u64,
    commitments: Vec<Hash>,
}

impl Channel {
    pub fn new(salt: &[u8]) -> Self {
        Self {
            current_hash: hash(salt),
            count: 0,
            commitments: Vec::new(),
        }
    }

    pub fn commit(&mut self, commitment: Hash) {
        self.commitments.push(commitment);

        let mut hasher = Hasher::new();
        hasher.update(self.current_hash.as_bytes());
        hasher.update(commitment.as_bytes());

        self.current_hash = hasher.finalize();
    }

    /// Draws a random element from `BaseField` (i.e. a number between 0 and 16).
    pub fn random_element(&mut self) -> BaseField {
        let hash_first_4_bytes: [u8; 4] = self.current_hash.as_bytes()[0..4].try_into().unwrap();
        let ret_element: BaseField = i32::from_le_bytes(hash_first_4_bytes).into();

        self.rehash_after_draw();

        ret_element
    }

    /// Draws a random integer (uniformly distributed) in the range [0, upper_bound-1].
    pub fn random_integer(&mut self, upper_bound: u8) -> u8 {
        let hash_first_byte: [u8; 1] = self.current_hash.as_bytes()[0..1].try_into().unwrap();
        let ret_element = u8::from_le_bytes(hash_first_byte) % upper_bound;

        self.rehash_after_draw();

        ret_element
    }

    /// this is an arbitrary way to change the current hash, so that we can call
    /// `random_element()` multiple times and always get a different one
    fn rehash_after_draw(&mut self) {
        let mut hasher = Hasher::new();
        hasher.update(self.current_hash.as_bytes());
        hasher.update(&self.count.to_le_bytes());

        self.count += 1;

        self.current_hash = hasher.finalize();
    }

    // Closes the channel, returning the commitments to be used in the final StarkProof
    pub fn finalize(self) -> Vec<Hash> {
        self.commitments
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
