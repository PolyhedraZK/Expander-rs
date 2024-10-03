use arith::{Field, FieldForECC, FieldSerde, FieldSerdeError};
use config::GKRConfig;
use std::{io::Read, vec};
use thiserror::Error;

use super::{Allocation, CoefType, Gate, RecursiveCircuit, Segment, Witness};
use crate::{GateAdd, GateConst, GateMul, SegmentId};

#[derive(Debug, Error)]
pub enum CircuitError {
    #[error("field serde error: {0:?}")]
    FieldSerdeError(#[from] FieldSerdeError),

    #[error("other error: {0:?}")]
    OtherError(#[from] std::io::Error),
}
pub trait FromEccSerde {
    fn deserialize_from<R: Read>(reader: R) -> Self;
}

impl<T: FromEccSerde> FromEccSerde for Vec<T> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let vec_len = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let mut ret = vec![];
        for _ in 0..vec_len {
            ret.push(T::deserialize_from(&mut reader));
        }
        ret
    }
}

impl<T1: FromEccSerde, T2: FromEccSerde> FromEccSerde for (T1, T2) {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        (
            T1::deserialize_from(&mut reader),
            T2::deserialize_from(&mut reader),
        )
    }
}

impl FromEccSerde for usize {
    fn deserialize_from<R: Read>(reader: R) -> Self {
        <usize as FieldSerde>::deserialize_from(reader).unwrap()
    }
}

impl<C: GKRConfig, const INPUT_NUM: usize> FromEccSerde for Gate<C, INPUT_NUM> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let mut i_ids = [0usize; INPUT_NUM];
        for id in &mut i_ids {
            *id = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        }

        let o_id = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();

        let coef_type_u8 = u8::deserialize_from(&mut reader).unwrap();
        let (coef_type, coef) = match coef_type_u8 {
            1 => (
                CoefType::Constant,
                C::CircuitField::try_deserialize_from_ecc_format(&mut reader).unwrap(),
            ),
            2 => (CoefType::Random, C::CircuitField::ZERO),
            3 => {
                if INPUT_NUM > 0 {
                    panic!("Public Input can only be used with constant gates")
                };

                (
                    CoefType::PublicInput(
                        <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
                    ),
                    C::CircuitField::ZERO,
                )
            }
            _ => unreachable!(),
        };

        Self {
            i_ids,
            o_id,
            coef_type,
            coef,
            gate_type: 0,
        }
    }
}

pub struct CustomGateWrapper<C: GKRConfig, const INPUT_NUM: usize> {
    pub custom_gate: Gate<C, INPUT_NUM>,
}

impl<C: GKRConfig, const INPUT_NUM: usize> FromEccSerde for CustomGateWrapper<C, INPUT_NUM> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let gate_type = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let i_ids: [usize; INPUT_NUM] = Vec::<usize>::deserialize_from(&mut reader)
            .try_into()
            .unwrap();

        let o_id = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();

        let coef_type_u8 = u8::deserialize_from(&mut reader).unwrap();
        let (coef_type, coef) = match coef_type_u8 {
            1 => (
                CoefType::Constant,
                C::CircuitField::try_deserialize_from_ecc_format(&mut reader).unwrap(),
            ),
            2 => (CoefType::Random, C::CircuitField::ZERO),
            3 => {
                if INPUT_NUM > 0 {
                    panic!("Public Input can only be used with constant gates")
                };

                (
                    CoefType::PublicInput(
                        <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
                    ),
                    C::CircuitField::ZERO,
                )
            }
            _ => unreachable!(),
        };

        Self {
            custom_gate: Gate::<C, INPUT_NUM> {
                i_ids,
                o_id,
                coef_type,
                coef,
                gate_type,
            },
        }
    }
}

impl FromEccSerde for Allocation {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        Self {
            i_offset: <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
            o_offset: <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
        }
    }
}

impl<C: GKRConfig> FromEccSerde for Segment<C> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let i_len = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let o_len = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        assert!(i_len.is_power_of_two());
        assert!(o_len.is_power_of_two());

        let child_segs = Vec::<(SegmentId, Vec<Allocation>)>::deserialize_from(&mut reader);
        let gate_muls = Vec::<GateMul<C>>::deserialize_from(&mut reader);
        let gate_adds = Vec::<GateAdd<C>>::deserialize_from(&mut reader);
        let gate_consts = Vec::<GateConst<C>>::deserialize_from(&mut reader);

        let mut gate_uni = vec![];
        let len = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        for _ in 0..len {
            let uni = CustomGateWrapper::<C, 1>::deserialize_from(&mut reader).custom_gate;
            gate_uni.push(uni);
        }

        Segment {
            i_var_num: i_len.trailing_zeros() as usize,
            o_var_num: o_len.trailing_zeros() as usize,
            child_segs,
            gate_muls,
            gate_adds,
            gate_consts,
            gate_uni,
        }
    }
}

const VERSION_NUM: usize = 3914834606642317635; // b'CIRCUIT6'

impl<C: GKRConfig> FromEccSerde for RecursiveCircuit<C> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let version_num = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        assert_eq!(version_num, VERSION_NUM);
        let expected_mod = <C::CircuitField as FieldForECC>::modulus();
        let mut field_mod = [0u8; 32];
        reader.read_exact(&mut field_mod).unwrap();
        let read_mod = ethnum::U256::from_le_bytes(field_mod);
        assert_eq!(expected_mod, read_mod);

        RecursiveCircuit {
            num_public_inputs: <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
            num_outputs: <usize as FieldSerde>::deserialize_from(&mut reader).unwrap(),
            expected_num_output_zeros: <usize as FieldSerde>::deserialize_from(&mut reader)
                .unwrap(),

            segments: Vec::<Segment<C>>::deserialize_from(&mut reader),
            layers: Vec::<usize>::deserialize_from(&mut reader),
        }
    }
}

impl<C: GKRConfig> FromEccSerde for Witness<C> {
    fn deserialize_from<R: Read>(mut reader: R) -> Self {
        let num_witnesses = <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let num_private_inputs_per_witness =
            <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let num_public_inputs_per_witness =
            <usize as FieldSerde>::deserialize_from(&mut reader).unwrap();
        let _modulus = <[u64; 4]>::deserialize_from(&mut reader).unwrap();

        let mut values = vec![];
        for _ in 0..num_witnesses * (num_private_inputs_per_witness + num_public_inputs_per_witness)
        {
            values.push(C::CircuitField::deserialize_from(&mut reader).unwrap());
        }

        Self {
            num_witnesses,
            num_private_inputs_per_witness,
            num_public_inputs_per_witness,
            values,
        }
    }
}
