use arith::{Field, SimdField};

use crate::{CircuitLayer, GKRConfig, GkrScratchpad};

#[inline(always)]
fn _eq<F: Field>(x: &F, y: &F) -> F {
    // x * y + (1 - x) * (1 - y)
    let xy = *x * y;
    xy + xy - x - y + F::from(1)
}

pub(crate) fn eq_evals_at_primitive<F: Field>(r: &[F], mul_factor: &F, eq_evals: &mut [F]) {
    eq_evals[0] = *mul_factor;
    let mut cur_eval_num = 1;

    for r_i in r.iter() {
        let eq_z_i_zero = F::one() - r_i;
        let eq_z_i_one = r_i;
        for j in 0..cur_eval_num {
            eq_evals[j + cur_eval_num] = eq_evals[j] * eq_z_i_one;
            eq_evals[j] *= eq_z_i_zero;
        }
        cur_eval_num <<= 1;
    }
}

pub(crate) fn eq_eval_at<F: Field>(
    r: &[F],
    mul_factor: &F,
    eq_evals: &mut [F],
    sqrt_n_1st: &mut [F],
    sqrt_n_2nd: &mut [F],
) {
    let first_half_bits = r.len() / 2;
    let first_half_mask = (1 << first_half_bits) - 1;
    eq_evals_at_primitive(&r[0..first_half_bits], mul_factor, sqrt_n_1st);
    eq_evals_at_primitive(&r[first_half_bits..], &F::one(), sqrt_n_2nd);

    for (i, eq_eval) in eq_evals.iter_mut().enumerate().take(1 << r.len()) {
        let first_half = i & first_half_mask;
        let second_half = i >> first_half_bits;
        *eq_eval = sqrt_n_1st[first_half] * sqrt_n_2nd[second_half];
    }
}

struct SumcheckMultilinearProdHelper {
    var_num: usize,
    sumcheck_var_idx: usize,
    cur_eval_size: usize,
}

impl SumcheckMultilinearProdHelper {
    fn new(var_num: usize) -> Self {
        SumcheckMultilinearProdHelper {
            var_num,
            sumcheck_var_idx: 0,
            cur_eval_size: 1 << var_num,
        }
    }

    fn poly_eval_at<C: GKRConfig>(
        &self,
        var_idx: usize,
        degree: usize,
        bk_f: &mut [C::Field],
        bk_hg: &mut [C::Field],
        init_v: &[C::SimdCircuitField],
        gate_exists: &[bool],
    ) -> [C::Field; 3] {
        assert_eq!(degree, 2);
        let mut p0 = C::Field::zero();
        let mut p1 = C::Field::zero();
        let mut p2 = C::Field::zero();
        log::trace!("bk_f: {:?}", &bk_f[..4]);
        log::trace!("bk_hg: {:?}", &bk_hg[..4]);
        log::trace!("init_v: {:?}", &init_v[..4]);

        let eval_size = 1 << (self.var_num - var_idx - 1);
        log::trace!("Eval size: {}", eval_size);

        if var_idx == 0 {
            for i in 0..eval_size {
                if !gate_exists[i * 2] && !gate_exists[i * 2 + 1] {
                    continue;
                }

                let f_v_0 = init_v[i * 2];
                let f_v_1 = init_v[i * 2 + 1];
                let hg_v_0 = bk_hg[i * 2];
                let hg_v_1 = bk_hg[i * 2 + 1];
                p0 += C::field_mul_simd_circuit_field(&hg_v_0, &f_v_0);
                log::trace!(
                    "p0.v+= {:?} * {:?} =  {:?}",
                    f_v_0,
                    hg_v_0,
                    C::field_mul_simd_circuit_field(&hg_v_0, &f_v_0) + p1
                );
                p1 += C::field_mul_simd_circuit_field(&hg_v_1, &f_v_1);
                p2 += C::field_mul_simd_circuit_field(&(hg_v_0 + hg_v_1), &(f_v_0 + f_v_1));
            }
        } else {
            for i in 0..eval_size {
                if !gate_exists[i * 2] && !gate_exists[i * 2 + 1] {
                    continue;
                }

                let f_v_0 = bk_f[i * 2];
                let f_v_1 = bk_f[i * 2 + 1];
                let hg_v_0 = bk_hg[i * 2];
                let hg_v_1 = bk_hg[i * 2 + 1];
                p0 += f_v_0 * hg_v_0;
                log::trace!(
                    "p0.v+= {:?} * {:?} =  {:?}",
                    f_v_0,
                    hg_v_0,
                    f_v_0 * hg_v_0 + p1
                );
                p1 += f_v_1 * hg_v_1;
                p2 += (f_v_0 + f_v_1) * (hg_v_0 + hg_v_1);
            }
        }
        p2 = p1.mul_by_6() + p0.mul_by_3() - p2.double();
        [p0, p1, p2]
    }

    fn receive_challenge<C: GKRConfig>(
        &mut self,
        var_idx: usize,
        r: C::ChallengeField,
        bk_f: &mut [C::Field],
        bk_hg: &mut [C::Field],
        init_v: &[C::SimdCircuitField],
        gate_exists: &mut [bool],
    ) {
        assert_eq!(var_idx, self.sumcheck_var_idx);
        assert!(var_idx < self.var_num);
        log::trace!("challenge eval size: {}", self.cur_eval_size);

        if var_idx == 0 {
            for i in 0..self.cur_eval_size >> 1 {
                if !gate_exists[i * 2] && !gate_exists[i * 2 + 1] {
                    gate_exists[i] = false;

                    bk_f[i] = C::field_add_simd_circuit_field(
                        &C::simd_circuit_field_mul_challenge_field(
                            &(init_v[2 * i + 1] - init_v[2 * i]),
                            &r,
                        ),
                        &init_v[2 * i],
                    );
                    bk_hg[i] = C::Field::zero();
                } else {
                    gate_exists[i] = true;

                    bk_f[i] = C::field_add_simd_circuit_field(
                        &C::simd_circuit_field_mul_challenge_field(
                            &(init_v[2 * i + 1] - init_v[2 * i]),
                            &r,
                        ),
                        &init_v[2 * i],
                    );
                    bk_hg[i] = bk_hg[2 * i] + (bk_hg[2 * i + 1] - bk_hg[2 * i]).scale(&r);
                }
            }
        } else {
            for i in 0..self.cur_eval_size >> 1 {
                if !gate_exists[i * 2] && !gate_exists[i * 2 + 1] {
                    gate_exists[i] = false;
                    bk_f[i] = bk_f[2 * i] + (bk_f[2 * i + 1] - bk_f[2 * i]).scale(&r);
                    bk_hg[i] = C::Field::zero();
                } else {
                    gate_exists[i] = true;
                    bk_f[i] = bk_f[2 * i] + (bk_f[2 * i + 1] - bk_f[2 * i]).scale(&r);
                    bk_hg[i] = bk_hg[2 * i] + (bk_hg[2 * i + 1] - bk_hg[2 * i]).scale(&r);
                }
            }
        }

        self.cur_eval_size >>= 1;
        self.sumcheck_var_idx += 1;
    }
}

#[allow(dead_code)]
pub(crate) struct SumcheckGkrHelper<'a, C: GKRConfig> {
    pub(crate) rx: Vec<C::ChallengeField>,
    pub(crate) ry: Vec<C::ChallengeField>,

    layer: &'a CircuitLayer<C>,
    sp: &'a mut GkrScratchpad<C>,
    rz0: &'a [C::ChallengeField],
    rz1: &'a [C::ChallengeField],
    alpha: C::ChallengeField,
    beta: C::ChallengeField,

    input_var_num: usize,
    output_var_num: usize,

    x_helper: SumcheckMultilinearProdHelper,
    y_helper: SumcheckMultilinearProdHelper,
}

impl<'a, C: GKRConfig> SumcheckGkrHelper<'a, C> {
    pub(crate) fn new(
        layer: &'a CircuitLayer<C>,
        rz0: &'a [C::ChallengeField],
        rz1: &'a [C::ChallengeField],
        alpha: &'a C::ChallengeField,
        beta: &'a C::ChallengeField,
        sp: &'a mut GkrScratchpad<C>,
    ) -> Self {
        SumcheckGkrHelper {
            rx: vec![],
            ry: vec![],

            layer,
            sp,
            rz0,
            rz1,
            alpha: *alpha,
            beta: *beta,

            input_var_num: layer.input_var_num,
            output_var_num: layer.output_var_num,

            x_helper: SumcheckMultilinearProdHelper::new(layer.input_var_num),
            y_helper: SumcheckMultilinearProdHelper::new(layer.input_var_num),
        }
    }

    pub(crate) fn poly_evals_at(&mut self, var_idx: usize, degree: usize) -> [C::Field; 3] {
        if var_idx < self.input_var_num {
            self.x_helper.poly_eval_at::<C>(
                var_idx,
                degree,
                &mut self.sp.v_evals,
                &mut self.sp.hg_evals,
                &self.layer.input_vals,
                &self.sp.gate_exists_5,
            )
        } else {
            self.y_helper.poly_eval_at::<C>(
                var_idx - self.input_var_num,
                degree,
                &mut self.sp.v_evals,
                &mut self.sp.hg_evals,
                &self.layer.input_vals,
                &self.sp.gate_exists_5,
            )
        }
    }

    pub(crate) fn receive_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        if var_idx < self.input_var_num {
            self.x_helper.receive_challenge::<C>(
                var_idx,
                r,
                &mut self.sp.v_evals,
                &mut self.sp.hg_evals,
                &self.layer.input_vals,
                &mut self.sp.gate_exists_5,
            );
            log::trace!("v_eval[0]:= {:?}", self.sp.v_evals[0]);
            self.rx.push(r);
        } else {
            self.y_helper.receive_challenge::<C>(
                var_idx - self.input_var_num,
                r,
                &mut self.sp.v_evals,
                &mut self.sp.hg_evals,
                &self.layer.input_vals,
                &mut self.sp.gate_exists_5,
            );
            self.ry.push(r);
        }
    }

    pub(crate) fn vx_claim(&self) -> C::Field {
        self.sp.v_evals[0]
    }

    pub(crate) fn vy_claim(&self) -> C::Field {
        self.sp.v_evals[0]
    }

    pub(crate) fn prepare_g_x_vals(&mut self) {
        let mul = &self.layer.mul;
        let add = &self.layer.add;
        let vals = &self.layer.input_vals;
        let eq_evals_at_rz0 = &mut self.sp.eq_evals_at_rz0;
        let eq_evals_at_rz1 = &mut self.sp.eq_evals_at_rz1;
        let gate_exists = &mut self.sp.gate_exists_5;
        let hg_vals = &mut self.sp.hg_evals;
        // hg_vals[0..vals.len()].fill(F::zero()); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(hg_vals.as_mut_ptr(), 0, vals.len());
        }
        // gate_exists[0..vals.len()].fill(false); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(gate_exists.as_mut_ptr(), 0, vals.len());
        }
        eq_eval_at(
            self.rz0,
            &self.alpha,
            eq_evals_at_rz0,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );
        eq_eval_at(
            self.rz1,
            &self.beta,
            eq_evals_at_rz1,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );
        for i in 0..1 << self.rz0.len() {
            eq_evals_at_rz0[i] += eq_evals_at_rz1[i];
        }

        for g in mul.iter() {
            let r = C::challenge_mul_circuit_field(&eq_evals_at_rz0[g.o_id], &g.coef);
            hg_vals[g.i_ids[0]] += C::simd_circuit_field_mul_challenge_field(&vals[g.i_ids[1]], &r);

            gate_exists[g.i_ids[0]] = true;
        }
        for g in add.iter() {
            hg_vals[g.i_ids[0]] += C::simd_circuit_field_mul_challenge_field(
                &g.coef,
                &eq_evals_at_rz0[g.o_id],
            );
            gate_exists[g.i_ids[0]] = true;
        }
    }

    pub(crate) fn prepare_h_y_vals(&mut self, v_rx: C::Field) {
        let mul = &self.layer.mul;
        let eq_evals_at_rz0 = &mut self.sp.eq_evals_at_rz0;
        let eq_evals_at_rx = &mut self.sp.eq_evals_at_rx;
        let gate_exists = &mut self.sp.gate_exists_5;
        let hg_vals = &mut self.sp.hg_evals;
        let fill_len = 1 << self.rx.len();
        // hg_vals[0..fill_len].fill(F::zero()); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(hg_vals.as_mut_ptr(), 0, fill_len);
        }
        // gate_exists[0..fill_len].fill(false); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(gate_exists.as_mut_ptr(), 0, fill_len);
        }

        eq_eval_at(
            &self.rx,
            &C::ChallengeField::one(),
            eq_evals_at_rx,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );

        for g in mul.iter() {
            hg_vals[g.i_ids[1]] += v_rx * 
                C::simd_circuit_field_mul_challenge_field(
                    &g.coef,
                    &(eq_evals_at_rz0[g.o_id] * eq_evals_at_rx[g.i_ids[0]]),
                );
            gate_exists[g.i_ids[1]] = true;
        }
    }
}
