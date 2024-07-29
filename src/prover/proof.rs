use std::io::{Read, Write};

use arith::{Field, FieldSerde};

/// Proof. In the serialized mode.
#[derive(Debug, Clone, Default)]
pub struct Proof {
    idx: usize,
    pub bytes: Vec<u8>,
}

impl Proof {
    #[inline(always)]
    pub fn append_u8_slice(&mut self, buffer: &[u8], size: usize) {
        self.bytes.extend_from_slice(&buffer[..size]);
    }

    #[inline(always)]
    pub fn step(&mut self, size: usize) {
        self.idx += size;
    }

    #[inline(always)]
    pub fn get_next_and_step<F: Field + FieldSerde>(&mut self) -> F {
        let ret = F::deserialize_from(&self.bytes[self.idx..]);
        self.step(F::SIZE);
        ret
    }
}

impl FieldSerde for Proof {
    #[inline(always)]
    fn serialize_into<W: Write>(&self, mut writer: W) {
        (self.bytes.len() as u64).serialize_into(&mut writer);
        writer.write_all(&self.bytes).unwrap();
    }

    #[inline(always)]
    fn serialized_size() -> usize {
        0 // proof is not a fixed size
    }

    #[inline(always)]
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let proof_len = u64::deserialize_from(&mut reader) as usize;
        let mut proof = vec![0u8; proof_len];
        reader.read_exact(&mut proof).unwrap();
        Self {
            idx: 0,
            bytes: proof,
        }
    }

    #[inline(always)]
    fn deserialize_from_ecc_format<R: Read>(_reader: R) -> Self {
        unimplemented!("not implemented for Proof")
    }

    fn try_deserialize_from_ecc_format<R: Read>(
        _reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!("not implemented for Proof")
    }
}
