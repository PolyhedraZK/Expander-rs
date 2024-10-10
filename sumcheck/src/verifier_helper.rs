use arith::{ExtensionField, Field};
use circuit::{CircuitLayer, CoefType, GateAdd, GateConst, GateMul};
use config::{FieldType, GKRConfig};
use polynomials::EqPolynomial;

use crate::{scratch_pad::VerifierScratchPad, unpack_and_combine};

#[derive(Default)]
pub struct GKRVerifierHelper {}

impl GKRVerifierHelper {
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn prepare_layer<C: GKRConfig>(
        layer: &CircuitLayer<C>,
        alpha: &C::ChallengeField,
        beta: &Option<C::ChallengeField>,
        rz0: &[C::ChallengeField],
        rz1: &Option<Vec<C::ChallengeField>>,
        r_simd: &Vec<C::ChallengeField>,
        r_mpi: &Vec<C::ChallengeField>,
        sp: &mut VerifierScratchPad<C>,
    ) {
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            rz0,
            alpha,
            &mut sp.eq_evals_at_rz0,
            &mut sp.eq_evals_first_part,
            &mut sp.eq_evals_second_part,
        );

        if beta.is_some() && rz1.is_some() {
            EqPolynomial::<C::ChallengeField>::eq_eval_at(
                rz1.as_ref().unwrap(),
                beta.as_ref().unwrap(),
                &mut sp.eq_evals_at_rz1,
                &mut sp.eq_evals_first_part,
                &mut sp.eq_evals_second_part,
            );

            for i in 0..(1usize << layer.output_var_num) {
                sp.eq_evals_at_rz0[i] += sp.eq_evals_at_rz1[i];
            }
        }

        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            r_simd,
            &C::ChallengeField::ONE,
            &mut sp.eq_evals_at_r_simd,
            &mut sp.eq_evals_first_part,
            &mut sp.eq_evals_second_part,
        );

        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            r_mpi,
            &C::ChallengeField::ONE,
            &mut sp.eq_evals_at_r_mpi,
            &mut sp.eq_evals_first_part,
            &mut sp.eq_evals_second_part,
        );

        sp.r_simd = r_simd;
        sp.r_mpi = r_mpi;
    }

    #[inline(always)]
    pub fn eval_cst<C: GKRConfig>(
        cst_gates: &[GateConst<C>],
        public_input: &[C::SimdCircuitField],
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        let mut v = C::ChallengeField::zero();

        let mpi_world_size = sp.eq_evals_at_r_mpi.len();
        let local_input_size = public_input.len() / mpi_world_size;

        let simd_sum: C::ChallengeField = sp.eq_evals_at_r_simd.iter().sum();
        let mpi_sum: C::ChallengeField = sp.eq_evals_at_r_mpi.iter().sum();
        let simd_mpi_sum = simd_sum * mpi_sum;

        for cst_gate in cst_gates {
            let tmp = match cst_gate.coef_type {
                CoefType::PublicInput(input_idx) => {
                    let mut input = vec![];
                    for i in 0..mpi_world_size {
                        input.push(public_input[i * local_input_size + input_idx]);
                    }

                    // mpi combined
                    let input_mpi_combined: C::Field = input
                        .iter()
                        .zip(&sp.eq_evals_at_r_mpi)
                        .map(|(v, c)| C::simd_circuit_field_mul_challenge_field(v, c))
                        .sum();

                    // simd combined
                    sp.eq_evals_at_rz0[cst_gate.o_id]
                        * unpack_and_combine::<C::Field>(
                            &input_mpi_combined,
                            &sp.eq_evals_at_r_simd,
                        )
                }
                _ => {
                    C::challenge_mul_circuit_field(
                        &sp.eq_evals_at_rz0[cst_gate.o_id],
                        &cst_gate.coef,
                    ) * simd_mpi_sum
                }
            };
            v += tmp;
        }

        v * simd_sum * mpi_sum
    }

    #[inline(always)]
    pub fn eval_add<C: GKRConfig>(
        add_gates: &[GateAdd<C>],
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        let mut v = C::ChallengeField::zero();
        for add_gate in add_gates {
            v += sp.eq_evals_at_rz0[add_gate.o_id]
                * C::challenge_mul_circuit_field(
                    &sp.eq_evals_at_rx[add_gate.i_ids[0]],
                    &add_gate.coef,
                );
        }
        v * sp.eq_r_simd_r_simd_xy * sp.eq_r_mpi_r_mpi_xy
    }

    #[inline(always)]
    pub fn eval_mul<C: GKRConfig>(
        mul_gates: &[GateMul<C>],
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        let mut v = C::ChallengeField::zero();
        for mul_gate in mul_gates {
            let tmp = sp.eq_evals_at_rx[mul_gate.i_ids[0]]
                * C::challenge_mul_circuit_field(
                    &sp.eq_evals_at_ry[mul_gate.i_ids[1]],
                    &mul_gate.coef,
                );
            v += sp.eq_evals_at_rz0[mul_gate.o_id] * tmp;
        }
        v * sp.eq_r_simd_r_simd_xy * sp.eq_r_mpi_r_mpi_xy
    }

    #[inline(always)]
    pub fn set_rx<C: GKRConfig>(rx: &[C::ChallengeField], sp: &mut VerifierScratchPad<C>) {
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            rx,
            &C::ChallengeField::ONE,
            &mut sp.eq_evals_at_rx,
            &mut sp.eq_evals_first_part,
            &mut sp.eq_evals_second_part,
        );
    }

    #[inline(always)]
    pub fn set_r_simd_xy<C: GKRConfig>(
        r_simd_xy: &[C::ChallengeField],
        sp: &mut VerifierScratchPad<C>,
    ) {
        sp.eq_r_simd_r_simd_xy = EqPolynomial::<C::ChallengeField>::eq_vec(
            unsafe { sp.r_simd.as_ref().unwrap() },
            r_simd_xy,
        );
    }

    #[inline(always)]
    pub fn set_r_mpi_xy<C: GKRConfig>(
        r_mpi_xy: &[C::ChallengeField],
        sp: &mut VerifierScratchPad<C>,
    ) {
        sp.eq_r_mpi_r_mpi_xy = EqPolynomial::<C::ChallengeField>::eq_vec(
            unsafe { sp.r_mpi.as_ref().unwrap() },
            r_mpi_xy,
        );
    }

    #[inline(always)]
    pub fn set_ry<C: GKRConfig>(ry: &[C::ChallengeField], sp: &mut VerifierScratchPad<C>) {
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            ry,
            &C::ChallengeField::ONE,
            &mut sp.eq_evals_at_ry,
            &mut sp.eq_evals_first_part,
            &mut sp.eq_evals_second_part,
        );
    }

    #[inline(always)]
    pub fn degree_2_eval<C: GKRConfig>(
        ps: &[C::ChallengeField],
        x: C::ChallengeField,
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        assert_eq!(ps.len(), 3);
        let p0 = ps[0];
        let p1 = ps[1];
        let p2 = ps[2];

        if C::FIELD_TYPE == FieldType::GF2 {
            let tmp = p0 - p1;

            let c0 = p0;
            let c2 = (p2 - p0 + tmp.mul_by_x()) * sp.gf2_deg2_eval_coef;
            let c1 = tmp - c2;
            c0 + (c2 * x + c1) * x
        } else {
            let c0 = p0;
            let c2 = C::ChallengeField::INV_2 * (p2 - p1 - p1 + p0);
            let c1 = p1 - p0 - c2;
            c0 + (c2 * x + c1) * x
        }
    }

    #[inline(always)]
    pub fn degree_3_eval<C: GKRConfig>(
        vals: &[C::ChallengeField],
        x: C::ChallengeField,
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        Self::lag_eval(vals, x, sp)
    }

    #[inline(always)]
    fn lag_eval<C: GKRConfig>(
        vals: &[C::ChallengeField],
        x: C::ChallengeField,
        sp: &VerifierScratchPad<C>,
    ) -> C::ChallengeField {
        assert_eq!(sp.deg3_eval_at.len(), vals.len());

        let mut v = C::ChallengeField::ZERO;
        for i in 0..vals.len() {
            let mut numerator = C::ChallengeField::ONE;
            for j in 0..vals.len() {
                if j == i {
                    continue;
                }
                numerator *= x - sp.deg3_eval_at[j];
            }
            v += numerator * sp.deg3_lag_denoms_inv[i] * vals[i];
        }
        v
    }
}
