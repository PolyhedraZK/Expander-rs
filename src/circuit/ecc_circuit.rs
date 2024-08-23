
use std::{collections::HashMap, fs, io::{Cursor, Read}};

use crate::GKRConfig;
use arith::{Field, FieldSerde};

use super::gate::*;
use super::expander_circuit::*;

const MAGIC_NUM: u64 = 3770719418566461763; // b'CIRCUIT4'

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
    pub gate_csts: Vec<GateConst<C>>,
    pub gate_unis: Vec<GateUni<C>>,
}

pub struct RecursiveCircuit<C: GKRConfig> {
    pub segments: Vec<Segment<C>>,
    pub layers: Vec<SegmentId>,
}


impl<C: GKRConfig> Segment<C> {
    pub fn contain_gates(&self) -> bool {
        !self.gate_muls.is_empty()
            || !self.gate_adds.is_empty()
            || !self.gate_csts.is_empty()
            || !self.gate_unis.is_empty()
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
            gate_csts: Vec::new(),
            gate_unis: Vec::new(),
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
            ret.gate_muls.push(GateMul::read(&mut reader));
        }

        let gate_adds_num = u64::deserialize_from(&mut reader) as usize;
        for _ in 0..gate_adds_num {
            ret.gate_adds.push(GateAdd::read(&mut reader));
        }

        let gate_consts_num = u64::deserialize_from(&mut reader) as usize;

        for _ in 0..gate_consts_num {
            ret.gate_csts.push(GateConst::read(&mut reader));
        }

        // TODO: process custom gate more properly
        let gate_custom_num = u64::deserialize_from(&mut reader) as usize;
        for _ in 0..gate_custom_num {
            let gate_type = u64::deserialize_from(&mut reader) as usize;
            let in_len = u64::deserialize_from(&mut reader) as usize;
            let mut inputs = Vec::new();
            for _ in 0..in_len {
                inputs.push(u64::deserialize_from(&mut reader) as usize);
            }
            let out = u64::deserialize_from(&mut reader) as usize;
            let coef_type = CoefType::read(&mut reader);
            
            let coef = if coef_type == CoefType::Constant {
                C::circuit_field_to_simd_circuit_field(&C::CircuitField::try_deserialize_from_ecc_format(&mut reader).unwrap())
            } else {
                C::SimdCircuitField::zero()
            };

            let gate = GateUni {
                i_ids: [inputs[0]],
                o_id: out,
                coef: coef,
                coef_type: coef_type,
                gate_type: gate_type,
            };
            ret.gate_unis.push(gate);
        }

        log::trace!(
            "gate nums: {} mul, {} add, {} const, {} custom",
            gate_muls_num,
            gate_adds_num,
            gate_consts_num,
            gate_custom_num
        );
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

        let field_mod = <[u64; 4]>::deserialize_from(&mut cursor);
        let num_public_inputs = u64::deserialize_from(&mut cursor);
        let num_actual_outputs = u64::deserialize_from(&mut cursor);
        let expected_num_output_zeros = u64::deserialize_from(&mut cursor);
        
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
                input_vals: vec![],
                output_vals: vec![],
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
                    for gate in &leaf_seg.gate_csts {
                        let mut gate = gate.clone();
                        gate.o_id += alloc.o_offset;
                        ret_layer.const_.push(gate);
                    }
                    for gate in &leaf_seg.gate_unis {
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

        ret.identify_special_coefs();
        ret
    }
}
