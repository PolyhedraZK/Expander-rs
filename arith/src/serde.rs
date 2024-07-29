use std::io::{Read, Write};

/// Serde for Fields
pub trait FieldSerde {
    /// serialize self into bytes
    fn serialize_into<W: Write>(&self, writer: W);

    /// size of the serialized bytes
    fn serialized_size() -> usize;

    /// deserialize bytes into field
    fn deserialize_from<R: Read>(reader: R) -> Self;

    /// deserialize bytes into field following ecc format
    fn deserialize_from_ecc_format<R: Read>(reader: R) -> Self;

    /// deserialize bytes into field following ecc format
    fn try_deserialize_from_ecc_format<R: Read>(
        reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized;
}

impl FieldSerde for u64 {
    /// serialize u64 into bytes
    fn serialize_into<W: Write>(&self, mut writer: W) {
        writer.write_all(&self.to_le_bytes()).unwrap();
    }

    /// size of the serialized bytes
    fn serialized_size() -> usize {
        8
    }

    /// deserialize bytes into u64
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer).unwrap();
        u64::from_le_bytes(buffer)
    }

    fn deserialize_from_ecc_format<R: Read>(_reader: R) -> Self {
        unimplemented!("not implemented for u64")
    }

    fn try_deserialize_from_ecc_format<R: Read>(
        _reader: R,
    ) -> std::result::Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!("not implemented for u64")
    }
}
