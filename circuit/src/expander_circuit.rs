use std::fs;
use std::io::Cursor;

use arith::{Field, SimdField};
use ark_std::test_rng;
use config::GKRConfig;
use transcript::Transcript;
use transcript::TranscriptInstance;

use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StructureInfo {
    pub max_degree_one: bool,
}

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

    pub structure_info: StructureInfo,
}

impl<C: GKRConfig> CircuitLayer<C> {
    pub fn evaluate(
        &self,
        res: &mut Vec<C::SimdCircuitField>,
        public_input: &[C::SimdCircuitField],
    ) {
        res.clear();
        res.resize(1 << self.output_var_num, C::SimdCircuitField::zero());
        for gate in &self.mul {
            let i0 = &self.input_vals[gate.i_ids[0]];
            let i1 = &self.input_vals[gate.i_ids[1]];
            let o = &mut res[gate.o_id];
            let mul = *i0 * i1;
            *o += C::circuit_field_mul_simd_circuit_field(&gate.coef, &mul);
        }

        for gate in &self.add {
            let i0 = self.input_vals[gate.i_ids[0]];
            let o = &mut res[gate.o_id];
            *o += C::circuit_field_mul_simd_circuit_field(&gate.coef, &i0);
        }

        for gate in &self.const_ {
            let o = &mut res[gate.o_id];

            let coef = match gate.coef_type {
                CoefType::PublicInput(input_idx) => public_input[input_idx],
                _ => C::circuit_field_to_simd_circuit_field(&gate.coef),
            };
            *o += coef;
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
                    *o += C::circuit_field_mul_simd_circuit_field(&gate.coef, &i0_5);
                }
                12346 => {
                    // pow1
                    *o += C::circuit_field_mul_simd_circuit_field(&gate.coef, i0);
                }
                _ => panic!("Unknown gate type: {}", gate.gate_type),
            }
        }
    }

    pub fn identify_rnd_coefs(&mut self, rnd_coefs: &mut Vec<*mut C::CircuitField>) {
        for gate in &mut self.mul {
            if gate.coef_type == CoefType::Random {
                rnd_coefs.push(&mut gate.coef);
            }
        }
        for gate in &mut self.add {
            if gate.coef_type == CoefType::Random {
                rnd_coefs.push(&mut gate.coef);
            }
        }
        for gate in &mut self.const_ {
            if gate.coef_type == CoefType::Random {
                rnd_coefs.push(&mut gate.coef);
            }
        }
        for gate in &mut self.uni {
            if gate.coef_type == CoefType::Random {
                rnd_coefs.push(&mut gate.coef);
            }
        }
    }

    pub fn identify_structure_info(&mut self) {
        self.structure_info.max_degree_one = self.mul.is_empty();
    }
}

#[derive(Debug, Default)]
pub struct Circuit<C: GKRConfig> {
    pub layers: Vec<CircuitLayer<C>>,
    pub public_input: Vec<C::SimdCircuitField>,
    pub expected_num_output_zeros: usize,

    pub rnd_coefs_identified: bool,
    pub rnd_coefs: Vec<*mut C::CircuitField>, // unsafe
}

impl<C: GKRConfig> Clone for Circuit<C> {
    fn clone(&self) -> Circuit<C> {
        let mut ret = Circuit::<C> {
            layers: self.layers.clone(),
            public_input: self.public_input.clone(),
            ..Default::default()
        };

        if self.rnd_coefs_identified {
            ret.identify_rnd_coefs();
        }
        ret
    }
}

unsafe impl<C> Send for Circuit<C> where C: GKRConfig {}

impl<C: GKRConfig> Circuit<C> {
    pub fn load_circuit(filename: &str) -> Self {
        let rc = RecursiveCircuit::<C>::load(filename).unwrap();
        rc.flatten()
    }

    pub fn load_witness_file(&mut self, filename: &str) {
        let file_bytes = fs::read(filename).unwrap();
        let cursor = Cursor::new(file_bytes);
        let witness = Witness::<C>::deserialize_from(cursor);

        let private_input_size = 1 << self.log_input_size();
        let public_input_size = witness.num_public_inputs_per_witness;
        let total_size = private_input_size + public_input_size;

        assert_eq!(witness.num_private_inputs_per_witness, private_input_size);
        #[allow(clippy::comparison_chain)]
        if witness.num_witnesses < C::get_field_pack_size() {
            panic!("Not enough witness");
        } else if witness.num_witnesses > C::get_field_pack_size() {
            println!("Warning: dropping additional witnesses");
        }

        let input = &witness.values;
        let private_input = &mut self.layers[0].input_vals;
        let public_input = &mut self.public_input;

        private_input.clear();
        public_input.clear();

        for i in 0..private_input_size {
            let mut private_wit_i = vec![];
            for j in 0..C::get_field_pack_size() {
                private_wit_i.push(input[j * total_size + i]);
            }
            private_input.push(C::SimdCircuitField::pack(&private_wit_i));
        }

        for i in 0..public_input_size {
            let mut public_wit_i = vec![];
            for j in 0..C::get_field_pack_size() {
                public_wit_i.push(input[j * total_size + private_input_size + i]);
            }
            public_input.push(C::SimdCircuitField::pack(&public_wit_i));
        }
    }
}

impl<C: GKRConfig> Circuit<C> {
    pub fn log_input_size(&self) -> usize {
        self.layers[0].input_var_num
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
                .evaluate(&mut layer_p_2[0].input_vals, &self.public_input);
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
        self.layers
            .last()
            .unwrap()
            .evaluate(&mut output, &self.public_input);
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

    pub fn identify_rnd_coefs(&mut self) {
        self.rnd_coefs.clear();
        for layer in &mut self.layers {
            layer.identify_rnd_coefs(&mut self.rnd_coefs);
        }
        self.rnd_coefs_identified = true;
    }

    pub fn fill_rnd_coefs(&mut self, transcript: &mut TranscriptInstance<C::FiatShamirHashType>) {
        assert!(self.rnd_coefs_identified);
        for &rnd_coef_ptr in &self.rnd_coefs {
            unsafe {
                *rnd_coef_ptr = transcript.generate_challenge::<C::CircuitField>();
            }
        }
    }

    pub fn identify_structure_info(&mut self) {
        for layer in &mut self.layers {
            layer.identify_structure_info();
        }
    }
}
