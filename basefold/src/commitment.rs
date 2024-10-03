use arith::{Field, FieldSerde};
use transcript::{FiatShamirHash, Transcript};
use tree::{Path, Tree};

#[derive(Debug, Clone, PartialEq)]
pub struct BasefoldCommitment<F: Field + FieldSerde> {
    pub(crate) tree: Tree<F>,
}

impl<F: Field + FieldSerde> BasefoldCommitment<F> {
    pub fn append_to_transcript<T, H>(&self, transcript: &mut T)
    where
        T: Transcript<H>,
        H: FiatShamirHash,
    {
        transcript.append_u8_slice(self.tree.root().as_bytes());
    }
}
