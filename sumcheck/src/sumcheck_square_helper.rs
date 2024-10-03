use arith::{Field, SimdField};
use circuit::CircuitLayer;
use config::GKRConfig;

use crate::{sumcheck_helper::eq_eval_at, GkrScratchpad};

struct SumcheckMultiSquareHelper<const D: usize> {
    var_num: usize,
    sumcheck_var_idx: usize,
    cur_eval_size: usize,
}

impl<const D: usize> SumcheckMultiSquareHelper<D> {
    fn new(var_num: usize) -> Self {
        SumcheckMultiSquareHelper {
            var_num,
            sumcheck_var_idx: 0,
            cur_eval_size: 1 << var_num,
        }
    }

    fn interpolate_3<C: GKRConfig>(p_add: &[C::Field; 3], p: &mut [C::Field; D]) {
        let p_add_coef_0 = p_add[0];
        let p_add_coef_2 = C::field_mul_circuit_field(
            &(p_add[2] - p_add[1] - p_add[1] + p_add[0]),
            &C::CircuitField::INV_2,
        );

        let p_add_coef_1 = p_add[1] - p_add_coef_0 - p_add_coef_2;

        p[0] += p_add_coef_0;
        p[1] += p_add_coef_0 + p_add_coef_1 + p_add_coef_2;
        p[2] += p_add_coef_0 + p_add_coef_1.double() + p_add_coef_2.double().double();
        p[3] += p_add_coef_0 + p_add_coef_1.mul_by_3() + p_add_coef_2.mul_by_3().mul_by_3();
        p[4] += p_add_coef_0
            + p_add_coef_1.double().double()
            + C::field_mul_circuit_field(&p_add_coef_2, &C::CircuitField::from(16));
        p[5] += p_add_coef_0
            + p_add_coef_1.mul_by_5()
            + C::field_mul_circuit_field(&p_add_coef_2, &C::CircuitField::from(25));
        p[6] += p_add_coef_0
            + p_add_coef_1.mul_by_3().double()
            + C::field_mul_circuit_field(&p_add_coef_2, &C::CircuitField::from(36));
    }

    #[allow(clippy::too_many_arguments)]
    fn poly_eval_at<C: GKRConfig>(
        &self,
        var_idx: usize,
        bk_f: &mut [C::Field],
        bk_hg_5: &mut [C::ChallengeField],
        bk_hg_1: &mut [C::ChallengeField],
        init_v: &[C::SimdCircuitField],
        gate_exists_5: &[bool],
        gate_exists_1: &[bool],
    ) -> [C::Field; D] {
        let mut p = [C::Field::zero(); D];
        log::trace!("bk_f: {:?}", &bk_f[..4]);
        log::trace!("bk_hg: {:?}", &bk_hg_5[..4]);
        log::trace!("init_v: {:?}", &init_v[..4]);
        if var_idx == 0 {
            let src_v = init_v;
            let eval_size = 1 << (self.var_num - var_idx - 1);
            log::trace!("Eval size: {}", eval_size);
            for i in 0..eval_size {
                if !gate_exists_5[i * 2] && !gate_exists_5[i * 2 + 1] {
                    continue;
                }
                let mut f_v = [C::SimdCircuitField::zero(); D];
                let mut hg_v = [C::ChallengeField::zero(); D];
                f_v[0] = src_v[i * 2];
                f_v[1] = src_v[i * 2 + 1];
                hg_v[0] = bk_hg_5[i * 2];
                hg_v[1] = bk_hg_5[i * 2 + 1];
                let delta_f = f_v[1] - f_v[0];
                let delta_hg = hg_v[1] - hg_v[0];

                for i in 2..D {
                    f_v[i] = f_v[i - 1] + delta_f;
                    hg_v[i] = hg_v[i - 1] + delta_hg;
                }
                for i in 0..D {
                    let pow5 = f_v[i].square().square() * f_v[i];
                    p[i] += C::simd_circuit_field_mul_challenge_field(&pow5, &hg_v[i]);
                }
            }
            let mut p_add = [C::Field::zero(); 3];
            for i in 0..eval_size {
                if !gate_exists_1[i * 2] && !gate_exists_1[i * 2 + 1] {
                    continue;
                }
                let mut f_v = [C::SimdCircuitField::zero(); 3];
                let mut hg_v = [C::ChallengeField::zero(); 3];
                f_v[0] = src_v[i * 2];
                f_v[1] = src_v[i * 2 + 1];
                hg_v[0] = bk_hg_1[i * 2];
                hg_v[1] = bk_hg_1[i * 2 + 1];
                p_add[0] += C::simd_circuit_field_mul_challenge_field(&f_v[0], &hg_v[0]);
                p_add[1] += C::simd_circuit_field_mul_challenge_field(&f_v[1], &hg_v[1]);
                let s_f_v = f_v[0] + f_v[1];
                let s_hg_v = hg_v[0] + hg_v[1];
                p_add[2] += C::simd_circuit_field_mul_challenge_field(&s_f_v, &s_hg_v);
            }

            p_add[2] = p_add[1].mul_by_6() + p_add[0].mul_by_3() - p_add[2].double();

            // interpolate p_add into 7 points
            Self::interpolate_3::<C>(&p_add, &mut p);
            p
        } else {
            let src_v = bk_f;
            let eval_size = 1 << (self.var_num - var_idx - 1);
            log::trace!("Eval size: {}", eval_size);
            for i in 0..eval_size {
                if !gate_exists_5[i * 2] && !gate_exists_5[i * 2 + 1] {
                    continue;
                }
                let mut f_v = [C::Field::zero(); D];
                let mut hg_v = [C::ChallengeField::zero(); D];
                f_v[0] = src_v[i * 2];
                f_v[1] = src_v[i * 2 + 1];
                hg_v[0] = bk_hg_5[i * 2];
                hg_v[1] = bk_hg_5[i * 2 + 1];
                let delta_f = f_v[1] - f_v[0];
                let delta_hg = hg_v[1] - hg_v[0];

                for i in 2..D {
                    f_v[i] = f_v[i - 1] + delta_f;
                    hg_v[i] = hg_v[i - 1] + delta_hg;
                }
                for i in 0..D {
                    let pow5 = f_v[i].square().square() * f_v[i];
                    p[i] += C::challenge_mul_field(&hg_v[i], &pow5);
                }
            }

            let mut p_add = [C::Field::zero(); 3];
            for i in 0..eval_size {
                if !gate_exists_1[i * 2] && !gate_exists_1[i * 2 + 1] {
                    continue;
                }
                let mut f_v = [C::Field::zero(); 3];
                let mut hg_v = [C::ChallengeField::zero(); 3];
                f_v[0] = src_v[i * 2];
                f_v[1] = src_v[i * 2 + 1];
                hg_v[0] = bk_hg_1[i * 2];
                hg_v[1] = bk_hg_1[i * 2 + 1];
                p_add[0] += C::challenge_mul_field(&hg_v[0], &f_v[0]);
                p_add[1] += C::challenge_mul_field(&hg_v[1], &f_v[1]);
            }
            p_add[2] = p_add[1] + p_add[1] - p_add[0] + C::Field::from(2);
            // interpolate p_add into 7 points
            Self::interpolate_3::<C>(&p_add, &mut p);
            p
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn receive_challenge<C: GKRConfig>(
        &mut self,
        var_idx: usize,
        r: C::ChallengeField,
        bk_f: &mut [C::Field],
        bk_hg_5: &mut [C::ChallengeField],
        bk_hg_1: &mut [C::ChallengeField],
        init_v: &[C::SimdCircuitField],
        gate_exists_5: &mut [bool],
        gate_exists_1: &mut [bool],
    ) {
        assert_eq!(var_idx, self.sumcheck_var_idx);
        assert!(var_idx < self.var_num);
        log::trace!("challenge eval size: {}", self.cur_eval_size);
        if var_idx == 0 {
            for i in 0..self.cur_eval_size >> 1 {
                let diff = init_v[2 * i + 1] - init_v[2 * i];
                let mul = C::simd_circuit_field_mul_challenge_field(&diff, &r);
                let init_v_0 = C::simd_circuit_field_into_field(&init_v[2 * i]);
                bk_f[i] = init_v_0 + mul;

                if !gate_exists_5[i * 2] && !gate_exists_5[i * 2 + 1] {
                    gate_exists_5[i] = false;
                    bk_hg_5[i] = C::ChallengeField::zero();
                } else {
                    gate_exists_5[i] = true;
                    bk_hg_5[i] = bk_hg_5[2 * i] + (bk_hg_5[2 * i + 1] - bk_hg_5[2 * i]) * r;
                }

                if !gate_exists_1[i * 2] && !gate_exists_1[i * 2 + 1] {
                    gate_exists_1[i] = false;
                    bk_hg_1[i] = C::ChallengeField::zero();
                } else {
                    gate_exists_1[i] = true;
                    bk_hg_1[i] = bk_hg_1[2 * i] + (bk_hg_1[2 * i + 1] - bk_hg_1[2 * i]) * r;
                }
            }
        } else {
            for i in 0..self.cur_eval_size >> 1 {
                bk_f[i] = bk_f[2 * i] + (bk_f[2 * i + 1] - bk_f[2 * i]).scale(&r);

                if !gate_exists_5[i * 2] && !gate_exists_5[i * 2 + 1] {
                    gate_exists_5[i] = false;
                    bk_hg_5[i] = C::ChallengeField::zero();
                } else {
                    gate_exists_5[i] = true;
                    bk_hg_5[i] = bk_hg_5[2 * i] + (bk_hg_5[2 * i + 1] - bk_hg_5[2 * i]) * r;
                }

                if !gate_exists_1[i * 2] && !gate_exists_1[i * 2 + 1] {
                    gate_exists_1[i] = false;
                    bk_hg_1[i] = C::ChallengeField::zero();
                } else {
                    gate_exists_1[i] = true;
                    bk_hg_1[i] = bk_hg_1[2 * i] + (bk_hg_1[2 * i + 1] - bk_hg_1[2 * i]) * r;
                }
            }
        }

        self.cur_eval_size >>= 1;
        self.sumcheck_var_idx += 1;
    }
}

#[allow(dead_code)]
// todo: Move D to GKRConfig
pub(crate) struct SumcheckGkrSquareHelper<'a, C: GKRConfig, const D: usize> {
    pub(crate) rx: Vec<C::ChallengeField>,

    layer: &'a CircuitLayer<C>,
    sp: &'a mut GkrScratchpad<C>,
    rz0: &'a [C::ChallengeField],

    input_var_num: usize,
    output_var_num: usize,

    x_helper: SumcheckMultiSquareHelper<D>,
}

impl<'a, C: GKRConfig, const D: usize> SumcheckGkrSquareHelper<'a, C, D> {
    pub(crate) fn new(
        layer: &'a CircuitLayer<C>,
        rz0: &'a [C::ChallengeField],
        sp: &'a mut GkrScratchpad<C>,
    ) -> Self {
        SumcheckGkrSquareHelper {
            rx: vec![],

            layer,
            sp,
            rz0,

            input_var_num: layer.input_var_num,
            output_var_num: layer.output_var_num,

            x_helper: SumcheckMultiSquareHelper::new(layer.input_var_num),
        }
    }

    #[inline]
    pub(crate) fn poly_evals_at(&mut self, var_idx: usize) -> [C::Field; D] {
        self.x_helper.poly_eval_at::<C>(
            var_idx,
            &mut self.sp.v_evals,
            &mut self.sp.hg_evals_5,
            &mut self.sp.hg_evals_1,
            &self.layer.input_vals,
            &self.sp.gate_exists_5,
            &self.sp.gate_exists_1,
        )
    }

    #[inline]
    pub(crate) fn receive_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.x_helper.receive_challenge::<C>(
            var_idx,
            r,
            &mut self.sp.v_evals,
            &mut self.sp.hg_evals_5,
            &mut self.sp.hg_evals_1,
            &self.layer.input_vals,
            &mut self.sp.gate_exists_5,
            &mut self.sp.gate_exists_1,
        );
        log::trace!("v_eval[0]:= {:?}", self.sp.v_evals[0]);
        self.rx.push(r);
    }

    #[inline(always)]
    pub(crate) fn vx_claim(&self) -> C::Field {
        self.sp.v_evals[0]
    }

    pub(crate) fn prepare_g_x_vals(&mut self) {
        let uni = &self.layer.uni; // univariate things like square, pow5, etc.
        let vals = &self.layer.input_vals;
        let eq_evals_at_rz0 = &mut self.sp.eq_evals_at_rz0;
        let gate_exists_5 = &mut self.sp.gate_exists_5;
        let gate_exists_1 = &mut self.sp.gate_exists_1;
        let hg_evals_5 = &mut self.sp.hg_evals_5;
        let hg_evals_1 = &mut self.sp.hg_evals_1;
        // hg_vals[0..vals.len()].fill(F::zero()); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(hg_evals_5.as_mut_ptr(), 0, vals.len());
            std::ptr::write_bytes(hg_evals_1.as_mut_ptr(), 0, vals.len());
        }
        // gate_exists[0..vals.len()].fill(false); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(gate_exists_5.as_mut_ptr(), 0, vals.len());
            std::ptr::write_bytes(gate_exists_1.as_mut_ptr(), 0, vals.len());
        }
        eq_eval_at(
            self.rz0,
            &C::ChallengeField::one(),
            eq_evals_at_rz0,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );

        for g in uni.iter() {
            match g.gate_type {
                12345 => {
                    hg_evals_5[g.i_ids[0]] +=
                        C::challenge_mul_circuit_field(&eq_evals_at_rz0[g.o_id], &g.coef);
                    gate_exists_5[g.i_ids[0]] = true;
                }
                12346 => {
                    hg_evals_1[g.i_ids[0]] +=
                        C::challenge_mul_circuit_field(&eq_evals_at_rz0[g.o_id], &g.coef);
                    gate_exists_1[g.i_ids[0]] = true;
                }
                _ => panic!("Unsupported gate type"),
            }
        }
    }
}
