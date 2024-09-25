use arith::{Field, FieldSerde};

use crate::{FiatShamirHash, GKRConfig, Proof};

pub struct Transcript<H: FiatShamirHash> {
    pub hasher: H,
    hash_start_idx: usize,
    digest: Vec<u8>,
    pub proof: Proof,
}

impl<H: FiatShamirHash> Default for Transcript<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: FiatShamirHash> Transcript<H> {
    pub const DIGEST_SIZE: usize = 32;

    #[inline]
    fn hash_to_digest(&mut self) {
        let hash_end_idx = self.proof.bytes.len();
        if hash_end_idx > self.hash_start_idx {
            self.hasher.hash(
                &mut self.digest,
                &self.proof.bytes[self.hash_start_idx..hash_end_idx],
            );
            self.hash_start_idx = hash_end_idx;
        } else {
            self.hasher.hash_inplace(&mut self.digest)
        }
    }

    #[inline]
    pub fn new() -> Self {
        Transcript {
            hasher: H::new(),
            hash_start_idx: 0,
            digest: vec![0u8; Self::DIGEST_SIZE],
            proof: Proof::default(),
        }
    }

    #[inline]
    pub fn append_f<C: GKRConfig>(&mut self, f: C::Field) {
        let cur_size = self.proof.bytes.len();
        self.proof.bytes.resize(cur_size + C::Field::SIZE, 0);
        f.serialize_into(&mut self.proof.bytes[cur_size..]).unwrap(); // TODO: error propagation
    }

    #[inline]
    pub fn append_challenge_f<C: GKRConfig>(&mut self, f: &C::ChallengeField) {
        let cur_size = self.proof.bytes.len();
        self.proof
            .bytes
            .resize(cur_size + C::ChallengeField::SIZE, 0);
        f.serialize_into(&mut self.proof.bytes[cur_size..]).unwrap();
    }

    #[inline]
    pub fn append_u8_slice(&mut self, buffer: &[u8]) {
        self.proof.bytes.extend_from_slice(buffer);
    }

    #[inline]
    pub fn challenge_f<C: GKRConfig>(&mut self) -> C::ChallengeField {
        self.hash_to_digest();
        assert!(C::ChallengeField::SIZE <= Self::DIGEST_SIZE);
        C::ChallengeField::from_uniform_bytes(&self.digest.clone().try_into().unwrap())
    }

    #[inline]
    pub fn challenge_fs<C: GKRConfig>(&mut self, size: usize) -> Vec<C::ChallengeField> {
        (0..size).map(|_| self.challenge_f::<C>()).collect()
    }

    #[inline]
    pub fn circuit_f<C: GKRConfig>(&mut self) -> C::CircuitField {
        self.hash_to_digest();
        assert!(C::CircuitField::SIZE <= Self::DIGEST_SIZE);
        C::CircuitField::from_uniform_bytes(&self.digest.clone().try_into().unwrap())
    }
}
