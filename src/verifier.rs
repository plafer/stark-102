use crate::{StarkProof, channel::Channel};


pub fn verify(proof: &StarkProof) -> anyhow::Result<()> {
    let mut channel = Channel::new();
    todo!()
}
