//! Scratch pad for prover and verifier to store intermediate values during the sumcheck protocol.

use std::{cmp::max, ptr};

use arith::{ExtensionField, Field};
use circuit::Circuit;
use config::{Config, FieldType, GKRConfig};

#[derive(Clone, Debug, Default)]
pub struct ProverScratchPad<C: GKRConfig> {
    pub v_evals: Vec<C::Field>,
    pub hg_evals_5: Vec<C::ChallengeField>,
    pub hg_evals_1: Vec<C::ChallengeField>,
    pub hg_evals: Vec<C::Field>,
    pub simd_var_v_evals: Vec<C::ChallengeField>,
    pub simd_var_hg_evals: Vec<C::ChallengeField>,
    pub mpi_var_v_evals: Vec<C::ChallengeField>,
    pub mpi_var_hg_evals: Vec<C::ChallengeField>,

    pub eq_evals_at_rx: Vec<C::ChallengeField>,
    pub eq_evals_at_rz0: Vec<C::ChallengeField>,
    pub eq_evals_at_rz1: Vec<C::ChallengeField>,
    pub eq_evals_at_r_simd0: Vec<C::ChallengeField>,
    pub eq_evals_at_r_mpi0: Vec<C::ChallengeField>,
    pub eq_evals_first_half: Vec<C::ChallengeField>,
    pub eq_evals_second_half: Vec<C::ChallengeField>,

    pub gate_exists_5: Vec<bool>,
    pub gate_exists_1: Vec<bool>,
}

impl<C: GKRConfig> ProverScratchPad<C> {
    pub fn new(max_num_input_var: usize, max_num_output_var: usize, mpi_world_size: usize) -> Self {
        let max_input_num = 1 << max_num_input_var;
        let max_output_num = 1 << max_num_output_var;
        ProverScratchPad {
            v_evals: vec![C::Field::default(); max_input_num],
            hg_evals_5: vec![C::ChallengeField::default(); max_input_num],
            hg_evals_1: vec![C::ChallengeField::default(); max_input_num],
            hg_evals: vec![C::Field::default(); max_input_num],
            simd_var_v_evals: vec![C::ChallengeField::default(); C::get_field_pack_size()],
            simd_var_hg_evals: vec![C::ChallengeField::default(); C::get_field_pack_size()],
            mpi_var_v_evals: vec![C::ChallengeField::default(); mpi_world_size],
            mpi_var_hg_evals: vec![C::ChallengeField::default(); mpi_world_size],

            eq_evals_at_rx: vec![C::ChallengeField::default(); max_input_num],
            eq_evals_at_rz0: vec![C::ChallengeField::default(); max_output_num],
            eq_evals_at_rz1: vec![C::ChallengeField::default(); max_output_num],
            eq_evals_at_r_simd0: vec![C::ChallengeField::default(); C::get_field_pack_size()],
            eq_evals_at_r_mpi0: vec![C::ChallengeField::default(); mpi_world_size],
            eq_evals_first_half: vec![C::ChallengeField::default(); max_output_num],
            eq_evals_second_half: vec![C::ChallengeField::default(); max_output_num],

            gate_exists_5: vec![false; max_input_num],
            gate_exists_1: vec![false; max_input_num],
        }
    }
}

pub struct VerifierScratchPad<C: GKRConfig> {
    // ====== for evaluating cst, add and mul ======
    pub eq_evals_at_rz0: Vec<C::ChallengeField>,
    pub eq_evals_at_rz1: Vec<C::ChallengeField>,
    pub eq_evals_at_r_simd: Vec<C::ChallengeField>,
    pub eq_evals_at_r_mpi: Vec<C::ChallengeField>,

    pub eq_evals_at_rx: Vec<C::ChallengeField>,
    pub eq_evals_at_ry: Vec<C::ChallengeField>,

    pub eq_evals_first_part: Vec<C::ChallengeField>,
    pub eq_evals_second_part: Vec<C::ChallengeField>,

    pub r_simd: *const Vec<C::ChallengeField>,
    pub r_mpi: *const Vec<C::ChallengeField>,
    pub eq_r_simd_r_simd_xy: C::ChallengeField,
    pub eq_r_mpi_r_mpi_xy: C::ChallengeField,

    // ====== for deg2, deg3 eval ======
    pub gf2_deg2_eval_coef: C::ChallengeField, // 1 / x(x - 1)
    pub deg3_eval_at: [C::ChallengeField; 4],
    pub deg3_lag_denoms_inv: [C::ChallengeField; 4],
}

impl<C: GKRConfig> VerifierScratchPad<C> {
    pub fn new(config: &Config<C>, circuit: &Circuit<C>) -> Self {
        let mut max_num_var = circuit
            .layers
            .iter()
            .map(|layer| layer.output_var_num)
            .max()
            .unwrap();
        max_num_var = max(max_num_var, circuit.log_input_size());

        let max_io_size = 1usize << max_num_var;
        let simd_size = C::get_field_pack_size();

        let gf2_deg2_eval_coef = if C::FIELD_TYPE == FieldType::GF2 {
            (C::ChallengeField::X - C::ChallengeField::one())
                .mul_by_x()
                .inv()
                .unwrap()
        } else {
            C::ChallengeField::INV_2
        };

        let deg3_eval_at = if C::FIELD_TYPE == FieldType::GF2 {
            [
                C::ChallengeField::ZERO,
                C::ChallengeField::ONE,
                C::ChallengeField::X,
                C::ChallengeField::X.mul_by_x(),
            ]
        } else {
            [
                C::ChallengeField::ZERO,
                C::ChallengeField::ONE,
                C::ChallengeField::from(2),
                C::ChallengeField::from(3),
            ]
        };

        let mut deg3_lag_denoms_inv = [C::ChallengeField::ZERO; 4];
        for i in 0..4 {
            let mut denominator = C::ChallengeField::ONE;
            for j in 0..4 {
                if j == i {
                    continue;
                }
                denominator *= deg3_eval_at[i] - deg3_eval_at[j];
            }
            deg3_lag_denoms_inv[i] = denominator.inv().unwrap();
        }

        Self {
            eq_evals_at_rz0: vec![C::ChallengeField::zero(); max_io_size],
            eq_evals_at_rz1: vec![C::ChallengeField::zero(); max_io_size],
            eq_evals_at_r_simd: vec![C::ChallengeField::zero(); simd_size],
            eq_evals_at_r_mpi: vec![C::ChallengeField::zero(); config.mpi_config.world_size()],

            eq_evals_at_rx: vec![C::ChallengeField::zero(); max_io_size],
            eq_evals_at_ry: vec![C::ChallengeField::zero(); max_io_size],

            eq_evals_first_part: vec![C::ChallengeField::zero(); max_io_size],
            eq_evals_second_part: vec![C::ChallengeField::zero(); max_io_size],

            r_simd: ptr::null(),
            r_mpi: ptr::null(),
            eq_r_simd_r_simd_xy: C::ChallengeField::zero(),
            eq_r_mpi_r_mpi_xy: C::ChallengeField::zero(),

            gf2_deg2_eval_coef,
            deg3_eval_at,
            deg3_lag_denoms_inv,
        }
    }
}
