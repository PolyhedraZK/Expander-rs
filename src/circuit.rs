use arith::{BinomialExtensionField, Field, FieldSerde, MultiLinearPoly};
use ark_std::test_rng;
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read},
};

use crate::{GKRConfig, Transcript};

#[derive(Debug, Clone)]
pub struct Gate<C: GKRConfig, const INPUT_NUM: usize> {
    pub i_ids: [usize; INPUT_NUM],
    pub o_id: usize,
    pub coef: C::CircuitField,
    pub gate_type: usize,
}

pub type GateMul<C> = Gate<C, 2>;
pub type GateAdd<C> = Gate<C, 1>;
pub type GateUni<C> = Gate<C, 1>;
pub type GateConst<C> = Gate<C, 0>;

#[derive(Debug, Clone, Default)]
pub struct CircuitLayer<C: GKRConfig> {
    pub input_var_num: usize,
    pub output_var_num: usize,

    pub input_vals: MultiLinearPoly<C::Field>,
    pub output_vals: MultiLinearPoly<C::Field>, // empty most time, unless in the last layer

    pub mul: Vec<GateMul<C>>,
    pub add: Vec<GateAdd<C>>,
    pub const_: Vec<GateConst<C>>,
    pub uni: Vec<GateUni<C>>,
}

impl<C: GKRConfig> CircuitLayer<C> {
    pub fn evaluate(&self) -> Vec<C::Field> {
        let mut res = vec![C::Field::zero(); 1 << self.output_var_num];
        for gate in &self.mul {
            let i0 = &self.input_vals.evals[gate.i_ids[0]];
            let i1 = &self.input_vals.evals[gate.i_ids[1]];
            let o = &mut res[gate.o_id];
            *o += C::field_mul_circuit_field(&(*i0 * i1), &gate.coef);
        }
        for gate in &self.add {
            let i0 = &self.input_vals.evals[gate.i_ids[0]];
            let o = &mut res[gate.o_id];
            *o += C::field_mul_circuit_field(i0, &gate.coef);
        }
        for gate in &self.const_ {
            let o = &mut res[gate.o_id];
            *o = C::field_add_circuit_field(o, &gate.coef);
        }
        for gate in &self.uni {
            let i0 = &self.input_vals.evals[gate.i_ids[0]];
            let o = &mut res[gate.o_id];
            match gate.gate_type {
                12345 => {
                    // pow5
                    let i0_2 = i0.square();
                    let i0_4 = i0_2.square();
                    let i0_5 = i0_4 * i0;
                    *o += C::field_mul_circuit_field(&i0_5, &gate.coef);
                }
                12346 => {
                    // pow1
                    *o += C::field_mul_circuit_field(i0, &gate.coef);
                }
                _ => panic!("Unknown gate type: {}", gate.gate_type),
            }
        }
        res
    }
}

#[derive(Debug, Clone, Default)]
pub struct Circuit<C: GKRConfig> {
    pub layers: Vec<CircuitLayer<C>>,
}

impl<C: GKRConfig> Circuit<C> {
    pub fn load_circuit(filename: &str) -> Self {
        let rc = RecursiveCircuit::<C>::load(filename);
        rc.flatten()
    }

    pub fn log_input_size(&self) -> usize {
        self.layers[0].input_var_num
    }

    // Build a random mock circuit with binary inputs
    pub fn set_random_bool_input_for_test(&mut self) {
        let mut rng = test_rng();
        self.layers[0].input_vals.evals = (0..(1 << self.log_input_size()))
            .map(|_| C::Field::random_bool(&mut rng))
            .collect();
    }

    pub fn evaluate(&mut self) {
        for i in 0..self.layers.len() - 1 {
            self.layers[i + 1].input_vals.evals = self.layers[i].evaluate();
            log::trace!(
                "layer {} evaluated - First 10 values: {:?}",
                i,
                self.layers[i + 1]
                    .input_vals
                    .evals
                    .iter()
                    .take(10)
                    .collect::<Vec<_>>()
            );
        }
        self.layers.last_mut().unwrap().output_vals.evals = self.layers.last().unwrap().evaluate();
        log::trace!("output evaluated");
        log::trace!(
            "First ten values: {:?}",
            self.layers
                .last()
                .unwrap()
                .output_vals
                .evals
                .iter()
                .take(10)
                .collect::<Vec<_>>()
        );
    }
}

// recursive format used in compiler
pub type SegmentId = usize;

pub struct Allocation {
    pub i_offset: usize,
    pub o_offset: usize,
}

pub struct Segment<C: GKRConfig> {
    pub i_var_num: usize,
    pub o_var_num: usize,
    pub child_segs: Vec<(SegmentId, Vec<Allocation>)>,
    pub gate_muls: Vec<GateMul<C>>,
    pub gate_adds: Vec<GateAdd<C>>,
    pub gate_consts: Vec<GateConst<C>>,
    pub gate_uni: Vec<GateUni<C>>,
}

impl<C: GKRConfig> Circuit<C> {
    pub fn load_witness_file(&mut self, filename: &str) {
        // note that, for data parallel, one should load multiple witnesses into different slot in the vectorized F
        let file_bytes = fs::read(filename).unwrap();
        self.load_witness_bytes(&file_bytes).unwrap();
    }
    pub fn load_witness_bytes(
        &mut self,
        file_bytes: &[u8],
    ) -> std::result::Result<(), std::io::Error> {
        log::trace!("witness file size: {} bytes", file_bytes.len());
        log::trace!("expecting: {} bytes", 32 * (1 << self.log_input_size()));
        if file_bytes.len() != 32 * (1 << self.log_input_size()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid witness file size",
            ));
        }

        let mut cursor = Cursor::new(file_bytes);
        let mut evals = vec![];
        for _ in 0..(1 << self.log_input_size()) {
            evals.push(
                match C::Field::try_deserialize_from_ecc_format(&mut cursor) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                },
            );
        }
        self.layers[0].input_vals.evals = evals;
        Ok(())
    }
}
impl<C: GKRConfig> Segment<C> {
    pub fn contain_gates(&self) -> bool {
        !self.gate_muls.is_empty()
            || !self.gate_adds.is_empty()
            || !self.gate_consts.is_empty()
            || !self.gate_uni.is_empty()
    }

    pub(crate) fn read<R: Read>(mut reader: R) -> Self {
        let i_len = u64::deserialize_from(&mut reader) as usize;
        let o_len = u64::deserialize_from(&mut reader) as usize;
        assert!(i_len.is_power_of_two());
        assert!(o_len.is_power_of_two());

        let mut ret = Segment::<C> {
            i_var_num: i_len.trailing_zeros() as usize,
            o_var_num: o_len.trailing_zeros() as usize,
            child_segs: Vec::new(),
            gate_muls: Vec::new(),
            gate_adds: Vec::new(),
            gate_consts: Vec::new(),
            gate_uni: Vec::new(),
        };

        let child_segs_num = u64::deserialize_from(&mut reader) as usize;

        for _ in 0..child_segs_num {
            let child_seg_id = u64::deserialize_from(&mut reader) as SegmentId;

            let allocation_num = u64::deserialize_from(&mut reader) as usize;

            for _ in 0..allocation_num {
                let i_offset = u64::deserialize_from(&mut reader) as usize;
                let o_offset = u64::deserialize_from(&mut reader) as usize;
                ret.child_segs
                    .push((child_seg_id, vec![Allocation { i_offset, o_offset }]));
            }
        }

        let gate_muls_num = u64::deserialize_from(&mut reader) as usize;
        for _ in 0..gate_muls_num {
            let gate = GateMul {
                i_ids: [
                    u64::deserialize_from(&mut reader) as usize,
                    u64::deserialize_from(&mut reader) as usize,
                ],
                o_id: u64::deserialize_from(&mut reader) as usize,
                coef: C::CircuitField::deserialize_from_ecc_format(&mut reader),
                gate_type: 0,
            };
            ret.gate_muls.push(gate);
        }

        let gate_adds_num = u64::deserialize_from(&mut reader) as usize;
        for _ in 0..gate_adds_num {
            let gate = GateAdd {
                i_ids: [u64::deserialize_from(&mut reader) as usize],
                o_id: u64::deserialize_from(&mut reader) as usize,

                coef: C::CircuitField::deserialize_from_ecc_format(&mut reader),
                gate_type: 1,
            };
            ret.gate_adds.push(gate);
        }
        let gate_consts_num = u64::deserialize_from(&mut reader) as usize;

        for _ in 0..gate_consts_num {
            let gate = GateConst {
                i_ids: [],
                o_id: u64::deserialize_from(&mut reader) as usize,

                coef: C::CircuitField::deserialize_from_ecc_format(&mut reader),
                gate_type: 2,
            };
            ret.gate_consts.push(gate);
        }

        let gate_custom_num = u64::deserialize_from(&mut reader) as usize;
        for _ in 0..gate_custom_num {
            let gate_type = u64::deserialize_from(&mut reader) as usize;
            let in_len = u64::deserialize_from(&mut reader) as usize;
            let mut inputs = Vec::new();
            for _ in 0..in_len {
                inputs.push(u64::deserialize_from(&mut reader) as usize);
            }
            let out = u64::deserialize_from(&mut reader) as usize;
            let coef = C::CircuitField::deserialize_from_ecc_format(&mut reader);
            let gate = GateUni {
                i_ids: [inputs[0]],
                o_id: out,
                coef,
                gate_type,
            };
            ret.gate_uni.push(gate);
        }

        log::trace!(
            "gate nums: {} mul, {} add, {} const, {} custom",
            gate_muls_num,
            gate_adds_num,
            gate_consts_num,
            gate_custom_num
        );

        let rand_coef_idx_num = u64::deserialize_from(&mut reader) as usize;
        let mut t = Transcript::new(); // FIXME LATER: use an empty transcript to align the randomness
        for _ in 0..rand_coef_idx_num {
            let idx = u64::deserialize_from(&mut reader) as usize;

            let rand_coef = t.challenge_f::<C>().first_base_field(); // ZZ: THIS SEEMS SUSPICIOUS
            if idx < ret.gate_muls.len() {
                ret.gate_muls[idx].coef = rand_coef;
            } else if idx < ret.gate_muls.len() + ret.gate_adds.len() {
                ret.gate_adds[idx - ret.gate_muls.len()].coef = rand_coef;
            } else if idx < ret.gate_muls.len() + ret.gate_adds.len() + ret.gate_consts.len() {
                ret.gate_consts[idx - ret.gate_muls.len() - ret.gate_adds.len()].coef = rand_coef;
            } else {
                ret.gate_uni
                    [idx - ret.gate_muls.len() - ret.gate_adds.len() - ret.gate_consts.len()]
                .coef = rand_coef;
            }
        }
        ret
    }

    pub fn scan_leaf_segments(
        &self,
        rc: &RecursiveCircuit<C>,
        cur_id: SegmentId,
    ) -> HashMap<SegmentId, Vec<Allocation>> {
        let mut ret = HashMap::new();
        if self.contain_gates() {
            ret.insert(
                cur_id,
                vec![Allocation {
                    i_offset: 0,
                    o_offset: 0,
                }],
            );
        }
        for (child_seg_id, child_allocs) in &self.child_segs {
            let leaves = rc.segments[*child_seg_id].scan_leaf_segments(rc, *child_seg_id);
            for (leaf_seg_id, leaf_allocs) in leaves {
                ret.entry(leaf_seg_id).or_insert_with(Vec::new);
                for child_alloc in child_allocs {
                    for leaf_alloc in &leaf_allocs {
                        ret.get_mut(&leaf_seg_id).unwrap().push(Allocation {
                            i_offset: child_alloc.i_offset + leaf_alloc.i_offset,
                            o_offset: child_alloc.o_offset + leaf_alloc.o_offset,
                        });
                    }
                }
            }
        }
        ret
    }
}

pub struct RecursiveCircuit<C: GKRConfig> {
    pub segments: Vec<Segment<C>>,
    pub layers: Vec<SegmentId>,
}

const MAGIC_NUM: u64 = 3770719418566461763; // b'CIRCUIT4'

impl<C: GKRConfig> RecursiveCircuit<C> {
    pub fn load(filename: &str) -> Self {
        let mut ret = RecursiveCircuit::<C> {
            segments: Vec::new(),
            layers: Vec::new(),
        };
        let file_bytes = fs::read(filename).unwrap();
        let mut cursor = Cursor::new(file_bytes);

        let magic_num = u64::deserialize_from(&mut cursor);
        assert_eq!(magic_num, MAGIC_NUM);

        let field_mod = C::Field::deserialize_from_ecc_format(&mut cursor);
        log::trace!("field mod: {:?}", field_mod);
        let segment_num = u64::deserialize_from(&mut cursor);
        for _ in 0..segment_num {
            let seg = Segment::<C>::read(&mut cursor);
            ret.segments.push(seg);
        }

        let layer_num = u64::deserialize_from(&mut cursor);
        for _ in 0..layer_num {
            let layer_id = u64::deserialize_from(&mut cursor) as SegmentId;

            ret.layers.push(layer_id);
        }
        ret
    }

    pub fn flatten(&self) -> Circuit<C> {
        let mut ret = Circuit::default();
        // layer-by-layer conversion
        for layer_id in &self.layers {
            let layer_seg = &self.segments[*layer_id];
            let leaves = layer_seg.scan_leaf_segments(self, *layer_id);
            let mut ret_layer = CircuitLayer {
                input_var_num: layer_seg.i_var_num,
                output_var_num: layer_seg.o_var_num,
                input_vals: MultiLinearPoly::<C::Field> {
                    var_num: layer_seg.i_var_num,
                    evals: vec![],
                },
                output_vals: MultiLinearPoly::<C::Field> {
                    var_num: layer_seg.o_var_num,
                    evals: vec![],
                },
                mul: vec![],
                add: vec![],
                const_: vec![],
                uni: vec![],
            };
            for (leaf_seg_id, leaf_allocs) in leaves {
                let leaf_seg = &self.segments[leaf_seg_id];
                for alloc in leaf_allocs {
                    for gate in &leaf_seg.gate_muls {
                        let mut gate = gate.clone();
                        gate.i_ids[0] += alloc.i_offset;
                        gate.i_ids[1] += alloc.i_offset;
                        gate.o_id += alloc.o_offset;
                        ret_layer.mul.push(gate);
                    }
                    for gate in &leaf_seg.gate_adds {
                        let mut gate = gate.clone();
                        gate.i_ids[0] += alloc.i_offset;
                        gate.o_id += alloc.o_offset;
                        ret_layer.add.push(gate);
                    }
                    for gate in &leaf_seg.gate_consts {
                        let mut gate = gate.clone();
                        gate.o_id += alloc.o_offset;
                        ret_layer.const_.push(gate);
                    }
                    for gate in &leaf_seg.gate_uni {
                        let mut gate = gate.clone();
                        gate.i_ids[0] += alloc.i_offset;
                        gate.o_id += alloc.o_offset;
                        ret_layer.uni.push(gate);
                    }
                }
            }
            // debug print layer
            log::trace!(
                "layer {} mul: {} add: {} const:{} uni:{} i_var_num: {} o_var_num: {}",
                ret.layers.len(),
                ret_layer.mul.len(),
                ret_layer.add.len(),
                ret_layer.const_.len(),
                ret_layer.uni.len(),
                ret_layer.input_var_num,
                ret_layer.output_var_num,
            );
            ret.layers.push(ret_layer);
        }

        ret
    }
}
