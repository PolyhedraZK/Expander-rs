use std::io::Read;
use arith::{Field, FieldSerde};
use crate::GKRConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoefType {
    Constant,
    Random,
    PublicInput(usize),
}

#[derive(Debug, Clone)]
pub struct Gate<C: GKRConfig, const INPUT_NUM: usize> {
    pub i_ids: [usize; INPUT_NUM],
    pub o_id: usize,
    pub coef: C::SimdCircuitField,
    pub coef_type: CoefType,
    pub gate_type: usize,
}

pub type GateMul<C> = Gate<C, 2>;
pub type GateAdd<C> = Gate<C, 1>;
pub type GateUni<C> = Gate<C, 1>;
pub type GateConst<C> = Gate<C, 0>;

impl CoefType {
    pub fn read<R: Read>(mut reader: R) -> Self {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte);
        
        if byte[0] == 1 {
            Self::Constant
        } else if byte[0] == 2 {
            Self::Random
        } else if byte[0] == 3 {
            Self::PublicInput(u64::deserialize_from(&mut reader) as usize)
        } else {
            unreachable!("Incorrect coef type")
        }
    }
}

impl<C: GKRConfig, const INPUT_NUM: usize> Gate<C, INPUT_NUM> {
    pub fn new() -> Self {
        Gate::<C, INPUT_NUM> {
            i_ids: [0; INPUT_NUM],
            o_id: 0,
            coef: C::SimdCircuitField::zero(),
            coef_type: CoefType::Constant,
            gate_type: 0,
        }
    }

    pub fn read<R: Read>(mut reader: R) -> Self {
        let mut gate = Self::new();

        for i in 0..INPUT_NUM {
            gate.i_ids[i] = u64::deserialize_from(&mut reader) as usize;
        }
        gate.o_id = u64::deserialize_from(&mut reader) as usize;
        gate.coef_type = CoefType::read(&mut reader);
        if gate.coef_type == CoefType::Constant {
            gate.coef = C::circuit_field_to_simd_circuit_field(
                &C::CircuitField::try_deserialize_from_ecc_format(reader).unwrap());
        }

        gate
    }
}