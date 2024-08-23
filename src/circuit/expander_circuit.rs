use std::cmp::min;
use std::{fs, io::Cursor};

use arith::{Field, FieldSerde, SimdField};
use ark_std::test_rng;
use crate::GKRConfig;
use crate::Transcript;

use super::gate::*;
use super::ecc_circuit::*;

#[derive(Debug, Clone, Default)]
pub struct CircuitLayer<C: GKRConfig> {
    pub input_var_num: usize,
    pub output_var_num: usize,

    pub input_vals: Vec<C::SimdCircuitField>,
    pub output_vals: Vec<C::SimdCircuitField>, // empty most time, unless in the last layer

    pub mul: Vec<GateMul<C>>,
    pub add: Vec<GateAdd<C>>,
    pub const_: Vec<GateConst<C>>,
    pub uni: Vec<GateUni<C>>,
}

#[derive(Debug, Default)]
pub struct Circuit<C: GKRConfig> {
    pub layers: Vec<CircuitLayer<C>>,

    // unsafe
    pub special_coefs_identified: bool,
    pub rnd_coefs: Vec<*mut C::SimdCircuitField>, 
    pub pub_coefs: Vec<(usize, *mut C::SimdCircuitField)>,
}

impl<C: GKRConfig> CircuitLayer<C> {
    pub fn evaluate(&self, res: &mut Vec<C::SimdCircuitField>) {
        res.clear();
        res.resize(1 << self.output_var_num, C::SimdCircuitField::zero());

        for gate in &self.mul {
            let i0 = &self.input_vals[gate.i_ids[0]];
            let i1 = &self.input_vals[gate.i_ids[1]];
            let o = &mut res[gate.o_id];
            let mul = *i0 * i1;
            *o += mul * gate.coef;
        }

        for gate in &self.add {
            let i0 = self.input_vals[gate.i_ids[0]];
            let o = &mut res[gate.o_id];
            *o += i0 * gate.coef;
        }
        
        for gate in &self.const_ {
            let o = &mut res[gate.o_id];
            *o += gate.coef;
        }
        
        for gate in &self.uni {
            let i0 = &self.input_vals[gate.i_ids[0]];
            let o = &mut res[gate.o_id];
            match gate.gate_type {
                12345 => {
                    // pow5
                    let i0_2 = i0.square();
                    let i0_4 = i0_2.square();
                    let i0_5 = i0_4 * i0;
                    *o += i0_5 * gate.coef;
                }
                12346 => {
                    // pow1
                    *o += *i0 * gate.coef;
                }
                _ => panic!("Unknown gate type: {}", gate.gate_type),
            }
        }
    }

    pub fn identify_special_coefs(
        &mut self,
        rnd_coefs: &mut Vec<*mut C::SimdCircuitField>,
        pub_coefs: &mut Vec<(usize, *mut C::SimdCircuitField)>,
    ) {
        macro_rules! collect_special_coefs {
            ($gate_type: ident) => {
                for gate in &mut self.$gate_type {
                    match(gate.coef_type) {
                        CoefType::Random => rnd_coefs.push(&mut gate.coef),
                        CoefType::PublicInput(idx) => pub_coefs.push((idx, &mut gate.coef)),
                        CoefType::Constant => () // do nothing,
                    }
                }
            };
        }

        collect_special_coefs!(add);
        collect_special_coefs!(mul);
        collect_special_coefs!(const_);
        collect_special_coefs!(uni);
    }

}

impl<C: GKRConfig> Clone for Circuit<C> {
    fn clone(&self) -> Circuit<C> {
        let mut ret = Circuit::<C> {
            layers: self.layers.clone(),
            special_coefs_identified: false,
            rnd_coefs: vec![],
            pub_coefs: vec![],
        };

        if self.special_coefs_identified {
            ret.identify_special_coefs();
        }
        ret
    }
}

unsafe impl<C: GKRConfig> Send for Circuit<C> {}

/// Serialization/Deserialization
impl<C: GKRConfig> Circuit<C> {
    pub fn load_circuit(filename: &str) -> Self {
        let rc = RecursiveCircuit::<C>::load(filename);
        rc.flatten()
    }
    
    pub fn load_witness_file(&mut self, filename: &str) {
        // note that, for data parallel, one should load multiple witnesses into different slot in the vectorized F
        let file_bytes = fs::read(filename).unwrap();
        self.load_witness_bytes(&file_bytes).unwrap();
    }

    /// vec2d: simd_size * n
    /// Returns: 1d simd vec of length n
    pub(crate) fn vec_2d_to_vec_simd(&self, vec2d: Vec<Vec<C::CircuitField>>) -> Vec<C::SimdCircuitField> {
        let simd_size = vec2d.len();
        let n = vec2d[0].len();

        let mut vec_simd = vec![];
        for i in 0..n {
            let mut vec_scalars = vec![];
            for j in 0..simd_size {
                vec_scalars.push(vec2d[j][i]);
            }
            vec_simd.push(C::simd_circuit_field_from_circuit_field_array(&vec_scalars));
        }

        vec_simd
    }

    pub fn load_witness_bytes(
        &mut self,
        file_bytes: &[u8],
    ) -> std::result::Result<(), std::io::Error> {
        log::trace!("witness file size: {} bytes", file_bytes.len());
        let mut cursor = Cursor::new(file_bytes);
        
        let num_witnesses = u64::deserialize_from(&mut cursor) as usize;
        let num_inputs_per_witness = u64::deserialize_from(&mut cursor) as usize;
        let num_public_inputs_per_witness = u64::deserialize_from(&mut cursor) as usize;
        let field_mod = <[u64; 4]>::deserialize_from(&mut cursor);
        log::trace!("Witness meta data: n_wit {}, n_inputs_per_wit {}, n_public_inputs_per_wit {}, field mod {:?}",
            num_witnesses, num_inputs_per_witness, num_public_inputs_per_witness, field_mod);

        let simd_size = C::SimdCircuitField::SIMD_SIZE;
        if num_witnesses !=  simd_size {
            println!("Num witness {} does not match simd size {}, padding/ignoring will occur", num_witnesses, simd_size);
        }

        let mut private_inputs = vec![vec![C::CircuitField::zero(); num_inputs_per_witness]; simd_size];
        let mut public_inputs = vec![vec![C::CircuitField::zero(); num_public_inputs_per_witness]; simd_size];

        // If simd_size > num_witnesses: read num_witnesses, leave all others 0
        // If num_witnesses > simd_size: read simd_size, ignore remaining witnesses
        let num_witnesses_to_read = min(num_witnesses, simd_size);
        for i in 0..num_witnesses_to_read {
            for j in 0..num_inputs_per_witness {
                private_inputs[i][j] = C::CircuitField::try_deserialize_from_ecc_format(&mut cursor)?;
            }

            for j in 0..num_public_inputs_per_witness {
                public_inputs[i][j] = C::CircuitField::try_deserialize_from_ecc_format(&mut cursor)?;
            }
        }

        let public_inputs_simd = self.vec_2d_to_vec_simd(public_inputs);
        self.fill_pub_coefs(&public_inputs_simd);

        self.layers[0].input_vals = self.vec_2d_to_vec_simd(private_inputs);

        
        Ok(())
    }
}


impl<C: GKRConfig> Circuit<C> {
    #[inline(always)]
    pub fn log_input_size(&self) -> usize {
        self.layers[0].input_var_num
    }

    #[inline(always)]
    pub fn input(&self) -> &Vec<C::SimdCircuitField> {
        return &self.layers[0].input_vals;
    }

    // Build a random mock circuit with binary inputs
    pub fn set_random_input_for_test(&mut self) {
        let mut rng = test_rng();
        self.layers[0].input_vals = (0..(1 << self.log_input_size()))
            .map(|_| C::SimdCircuitField::random_unsafe(&mut rng))
            .collect();
    }

    pub fn evaluate(&mut self) {
        for i in 0..self.layers.len() - 1 {
            let (layer_p_1, layer_p_2) = self.layers.split_at_mut(i + 1);
            layer_p_1
                .last()
                .unwrap()
                .evaluate(&mut layer_p_2[0].input_vals);
            log::trace!(
                "layer {} evaluated - First 10 values: {:?}",
                i,
                self.layers[i + 1]
                    .input_vals
                    .iter()
                    .take(10)
                    .collect::<Vec<_>>()
            );
        }
        let mut output = vec![];
        self.layers.last().unwrap().evaluate(&mut output);
        self.layers.last_mut().unwrap().output_vals = output;

        log::trace!("output evaluated");
        log::trace!(
            "First ten values: {:?}",
            self.layers
                .last()
                .unwrap()
                .output_vals
                .iter()
                .take(10)
                .collect::<Vec<_>>()
        );
    }

    pub fn identify_special_coefs(&mut self) {
        self.rnd_coefs.clear();
        self.pub_coefs.clear();
        for layer in &mut self.layers {
            layer.identify_special_coefs(&mut self.rnd_coefs, &mut self.pub_coefs);
        }
        self.special_coefs_identified = true;
    }

    pub fn fill_rnd_coefs(&mut self, transcript: &mut Transcript) {
        assert!(self.special_coefs_identified);
        for &rnd_coef_ptr in &self.rnd_coefs {
            unsafe {
                *rnd_coef_ptr = C::circuit_field_to_simd_circuit_field(&transcript.circuit_f::<C>());
            }
        }
    }

    pub fn fill_pub_coefs(&mut self, public_inputs: &[C::SimdCircuitField]) {
        assert!(self.special_coefs_identified);
        for &(idx, pub_coef_ptr) in &self.pub_coefs {
            unsafe {
                *pub_coef_ptr = public_inputs[idx];
            }
        }
    }

}

