use std::{fmt::Debug, marker::PhantomData};

use arith::{Field, FieldSerde};

use crate::{fiat_shamir_hash::FiatShamirHash, Proof};

pub trait Transcript<H: FiatShamirHash>: Debug + Clone {
    /// Create a new transcript.
    fn new() -> Self;

    /// Append a field element to the transcript.
    fn append_field_element<F>(&mut self, f: &F)
    where
        F: FieldSerde,
    {
        let mut buf = vec![];
        f.serialize_into(&mut buf).unwrap();
        self.append_u8_slice(&buf);
    }

    /// Append a byte slice to the transcript.
    fn append_u8_slice(&mut self, buffer: &[u8]);

    /// Generate a challenge field element.
    fn generate_challenge<F: Field>(&mut self) -> F;

    /// Generate a challenge vector.
    #[inline]
    fn generate_challenge_vector<F: Field>(&mut self, n: usize) -> Vec<F> {
        let mut challenges = Vec::with_capacity(n);
        for _ in 0..n {
            challenges.push(self.generate_challenge());
        }
        challenges
    }

    /// Generate a usize as a challenge index.  
    fn generate_challenge_index(&mut self) -> usize;

    /// Generate a vector of usize as a challenge index vector.  
    fn generate_challenge_index_vector(&mut self, n: usize) -> Vec<usize>;
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TranscriptInstance<H: FiatShamirHash> {
    phantom: PhantomData<H>,

    /// The digest bytes.
    pub digest: Vec<u8>,

    /// The proof bytes.
    pub proof: Proof,

    /// The pointer to the proof bytes indicating where the hash starts.
    hash_start_index: usize,
}

impl<H: FiatShamirHash> Transcript<H> for TranscriptInstance<H> {
    #[inline]
    fn new() -> Self {
        TranscriptInstance {
            phantom: PhantomData,
            digest: vec![0u8; H::DIGEST_SIZE],
            proof: Proof::default(),
            hash_start_index: 0,
        }
    }

    /// Append a byte slice to the transcript.
    #[inline]
    fn append_u8_slice(&mut self, buffer: &[u8]) {
        self.proof.bytes.extend_from_slice(buffer);
    }

    /// Generate a challenge.
    #[inline]
    fn generate_challenge<F: Field>(&mut self) -> F {
        self.hash_to_digest();
        assert!(F::SIZE <= H::DIGEST_SIZE);
        F::from_uniform_bytes(&self.digest.clone().try_into().unwrap())
    }

    /// Generate a usize as a challenge index.
    #[inline]
    fn generate_challenge_index(&mut self) -> usize {
        self.hash_to_digest();
        let mut index_bytes = [0u8; std::mem::size_of::<usize>()];
        index_bytes.copy_from_slice(&self.digest[..std::mem::size_of::<usize>()]);
        usize::from_le_bytes(index_bytes)
    }

    #[inline]
    fn generate_challenge_index_vector(&mut self, n: usize) -> Vec<usize> {
        let hashes = n / 4;
        let mut res = Vec::with_capacity(n);
        let reminder = n % 4;
        for _ in 0..hashes - 1 {
            self.hash_to_digest();

            for i in 0..4 {
                let mut index_bytes = [0u8; std::mem::size_of::<usize>()];
                index_bytes.copy_from_slice(&self.digest[i * 8..(i + 1) * 8]);
                res.push(usize::from_le_bytes(index_bytes));
            }
        }

        self.hash_to_digest();

        for i in 0..reminder {
            let mut index_bytes = [0u8; std::mem::size_of::<usize>()];
            index_bytes.copy_from_slice(&self.digest[i * 8..(i + 1) * 8]);
            res.push(usize::from_le_bytes(index_bytes));
        }

        res
    }
}

impl<H: FiatShamirHash> TranscriptInstance<H> {
    /// Hash the input into the output.
    pub fn hash_to_digest(&mut self) {
        let hash_end_index = self.proof.bytes.len();
        if hash_end_index > self.hash_start_index {
            H::hash(
                &mut self.digest,
                &self.proof.bytes[self.hash_start_index..hash_end_index],
            );
            self.hash_start_index = hash_end_index;
        } else {
            H::hash_inplace(&mut self.digest);
        }
    }
}
