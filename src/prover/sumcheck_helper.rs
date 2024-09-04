use arith::{ExtensionField, Field, SimdField};

use crate::{CircuitLayer, FieldType, GKRConfig, GkrScratchpad};

#[inline(always)]
fn _eq<F: Field>(x: &F, y: &F) -> F {
    // x * y + (1 - x) * (1 - y)
    let xy = *x * y;
    xy + xy - x - y + F::one()
}

#[inline(always)]
fn _eq_3<F: Field>(x: &F, y: &F, z: &F) -> F {
    // TODO: extend this
    *x * y * z + (F::one() - x) * (F::one() - y) * (F::one() - z)
}

#[inline(always)]
pub(crate) fn _eq_vec<F: Field>(xs: &[F], ys: &[F]) -> F {
    xs.iter().zip(ys.iter()).map(|(x, y)| _eq(x, y)).product()
}

#[inline(always)]
pub(crate) fn _eq_vec_3<F: Field>(xs: &[F], ys: &[F], zs: &[F]) -> F {
    xs.iter()
        .zip(ys.iter())
        .zip(zs.iter())
        .map(|((x, y), z)| _eq_3(x, y, z))
        .product()
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
}

impl SumcheckMultilinearProdHelper {
    fn new(var_num: usize) -> Self {
        SumcheckMultilinearProdHelper { var_num }
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
        if C::FIELD_TYPE == FieldType::GF2 {
            let p2x = p2.mul_by_x();
            let p2x2 = p2x.mul_by_x();
            let linear_term = p1 + p0 + p2;
            p2 = p2x2 + linear_term.mul_by_x() + p0;
        } else {
            p2 = p1.mul_by_6() + p0.mul_by_3() - p2.double();
        }
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
        assert!(var_idx < self.var_num);

        let eval_size = 1 << (self.var_num - var_idx - 1);
        if var_idx == 0 {
            for i in 0..eval_size {
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
            for i in 0..eval_size {
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
    }
}

struct SumcheckMultilinearProdSimdVarHelper {
    var_num: usize,
}

// The logic is exactly the same as the previous one, but field types are different
impl SumcheckMultilinearProdSimdVarHelper {
    fn new(var_num: usize) -> Self {
        SumcheckMultilinearProdSimdVarHelper { var_num }
    }

    fn poly_eval_at<C: GKRConfig>(
        &self,
        var_idx: usize,
        degree: usize,
        bk_f: &mut [C::ChallengeField],
        bk_hg: &mut [C::ChallengeField],
    ) -> [C::ChallengeField; 3] {
        assert_eq!(degree, 2);
        let mut p0 = C::ChallengeField::zero();
        let mut p1 = C::ChallengeField::zero();
        let mut p2 = C::ChallengeField::zero();

        let eval_size = 1 << (self.var_num - var_idx - 1);

        for i in 0..eval_size {
            let f_v_0 = bk_f[i * 2];
            let f_v_1 = bk_f[i * 2 + 1];
            let hg_v_0 = bk_hg[i * 2];
            let hg_v_1 = bk_hg[i * 2 + 1];
            p0 += f_v_0 * hg_v_0;

            p1 += f_v_1 * hg_v_1;
            p2 += (f_v_0 + f_v_1) * (hg_v_0 + hg_v_1);
        }

        if C::FIELD_TYPE == FieldType::GF2 {
            let p2x = p2.mul_by_x();
            let p2x2 = p2x.mul_by_x();
            let linear_term = p1 + p0 + p2;
            p2 = p2x2 + linear_term.mul_by_x() + p0;
        } else {
            p2 = p1.mul_by_6() + p0.mul_by_3() - p2.double();
        }
        [p0, p1, p2]
    }

    fn receive_challenge<C: GKRConfig>(
        &mut self,
        var_idx: usize,
        r: C::ChallengeField,
        bk_f: &mut [C::ChallengeField],
        bk_hg: &mut [C::ChallengeField],
    ) {
        assert!(var_idx < self.var_num);

        let eval_size = 1 << (self.var_num - var_idx - 1);
        for i in 0..eval_size {
            bk_f[i] = bk_f[2 * i] + (bk_f[2 * i + 1] - bk_f[2 * i]) * (&r);
            bk_hg[i] = bk_hg[2 * i] + (bk_hg[2 * i + 1] - bk_hg[2 * i]) * (&r);
        }
    }
}

pub(crate) struct SumcheckGkrHelper<'a, C: GKRConfig> {
    pub(crate) rx: Vec<C::ChallengeField>,
    pub(crate) ry: Vec<C::ChallengeField>,
    pub(crate) r_simd_var_x: Vec<C::ChallengeField>,
    pub(crate) r_simd_var_y: Vec<C::ChallengeField>,

    layer: &'a CircuitLayer<C>,
    sp: &'a mut GkrScratchpad<C>,
    rz0: &'a [C::ChallengeField],
    rz1: &'a [C::ChallengeField],
    r_simd0: &'a [C::ChallengeField],
    r_simd1: &'a [C::ChallengeField],
    alpha: C::ChallengeField,
    beta: C::ChallengeField,

    pub(crate) input_var_num: usize,
    output_var_num: usize,
    pub(crate) simd_var_num: usize,

    xy_helper: SumcheckMultilinearProdHelper,
    simd_var_helper: SumcheckMultilinearProdSimdVarHelper,
}

/// internal helper functions
impl<'a, C: GKRConfig> SumcheckGkrHelper<'a, C> {
    #[inline(always)]
    pub(crate) fn unpack_and_sum(p: C::Field) -> C::ChallengeField {
        p.unpack().into_iter().sum()
    }

    pub(crate) fn unpack_and_combine(p: C::Field, coef: &[C::ChallengeField]) -> C::ChallengeField {
        let p_unpacked = p.unpack();
        p_unpacked
            .into_iter()
            .zip(coef)
            .map(|(p_i, coef_i)| p_i * coef_i)
            .sum()
    }

    #[inline(always)]
    fn xy_helper_receive_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.xy_helper.receive_challenge::<C>(
            var_idx,
            r,
            &mut self.sp.v_evals,
            &mut self.sp.hg_evals,
            &self.layer.input_vals,
            &mut self.sp.gate_exists_5,
        );
    }

    #[inline(always)]
    fn simd_helper_receive_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.simd_var_helper.receive_challenge::<C>(
            var_idx,
            r,
            &mut self.sp.simd_var_v_evals,
            &mut self.sp.simd_var_hg_evals,
        );
    }
}

/// Helper functions to be called
impl<'a, C: GKRConfig> SumcheckGkrHelper<'a, C> {
    pub(crate) fn new(
        layer: &'a CircuitLayer<C>,
        rz0: &'a [C::ChallengeField],
        rz1: &'a [C::ChallengeField],
        r_simd0: &'a [C::ChallengeField],
        r_simd1: &'a [C::ChallengeField],
        alpha: &'a C::ChallengeField,
        beta: &'a C::ChallengeField,
        sp: &'a mut GkrScratchpad<C>,
    ) -> Self {
        let simd_var_num = C::get_field_pack_size().trailing_zeros() as usize;
        SumcheckGkrHelper {
            rx: vec![],
            ry: vec![],
            r_simd_var_x: vec![],
            r_simd_var_y: vec![],

            layer,
            sp,
            rz0,
            rz1,
            r_simd0,
            r_simd1,
            alpha: *alpha,
            beta: *beta,

            input_var_num: layer.input_var_num,
            output_var_num: layer.output_var_num,
            simd_var_num,

            xy_helper: SumcheckMultilinearProdHelper::new(layer.input_var_num),
            simd_var_helper: SumcheckMultilinearProdSimdVarHelper::new(simd_var_num),
        }
    }

    pub(crate) fn poly_evals_at_rx(
        &mut self,
        var_idx: usize,
        degree: usize,
    ) -> [C::ChallengeField; 3] {
        debug_assert!(var_idx < self.input_var_num);
        let [p0, p1, p2] = self.xy_helper.poly_eval_at::<C>(
            var_idx,
            degree,
            &mut self.sp.v_evals,
            &mut self.sp.hg_evals,
            &self.layer.input_vals,
            &self.sp.gate_exists_5,
        );
        [
            Self::unpack_and_sum(p0),
            Self::unpack_and_sum(p1),
            Self::unpack_and_sum(p2),
        ]
    }

    pub(crate) fn poly_evals_at_r_simd_var_x(
        &mut self,
        var_idx: usize,
        degree: usize,
    ) -> [C::ChallengeField; 3] {
        debug_assert!(var_idx < self.simd_var_num);
        self.simd_var_helper.poly_eval_at::<C>(
            var_idx,
            degree,
            &mut self.sp.simd_var_v_evals,
            &mut self.sp.simd_var_hg_evals,
        )
    }

    #[inline(always)]
    pub(crate) fn poly_evals_at_ry(
        &mut self,
        var_idx: usize,
        degree: usize,
    ) -> [C::ChallengeField; 3] {
        self.poly_evals_at_rx(var_idx, degree)
    }

    #[inline(always)]
    pub(crate) fn poly_evals_at_simd_var_y(
        &mut self,
        var_idx: usize,
        degree: usize,
    ) -> [C::ChallengeField; 3] {
        self.poly_evals_at_r_simd_var_x(var_idx, degree)
    }

    pub(crate) fn receive_rx(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.xy_helper_receive_challenge(var_idx, r);
        self.rx.push(r);
    }

    pub(crate) fn receive_r_simd_var_x(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.simd_helper_receive_challenge(var_idx, r);
        self.r_simd_var_x.push(r);
    }

    pub(crate) fn receive_ry(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.xy_helper_receive_challenge(var_idx, r);
        self.ry.push(r);
    }

    pub(crate) fn receive_r_simd_var_y(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.simd_helper_receive_challenge(var_idx, r);
        self.r_simd_var_y.push(r);
    }

    /// Warning:
    /// The function must be called at a specific point of the protocol, otherwise it's incorrect
    /// Consider fix this.
    pub(crate) fn vx_claim(&self) -> C::ChallengeField {
        self.sp.simd_var_v_evals[0]
    }

    /// Seems wierd, but it's correct since this is called at a different time of the protocol
    #[inline(always)]
    pub(crate) fn vy_claim(&self) -> C::ChallengeField {
        self.vx_claim()
    }

    pub(crate) fn prepare_simd(&mut self) {
        eq_eval_at(
            &self.r_simd0,
            &C::ChallengeField::one(),
            &mut self.sp.eq_evals_at_r_simd0,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );

        eq_eval_at(
            &self.r_simd1,
            &C::ChallengeField::one(),
            &mut self.sp.eq_evals_at_r_simd1,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );
    }

    /// V(rz, rw)
    /// = \sum_{x, wx, y, wy} eq(w, wx) add(rz, x) V(x, wx) + eq(w, wx, wy) mul(rz, x, y) V(x, wx) V(y, wy)
    /// = \sum_{x, wx, y, wy} V(x, wx) (eq(w, wx) add(rz, w) + eq(w, wx, wy) mul(rz, x, y) V(y, wy))
    /// = \sum_{x, wx, y, wy} V(x, wx) g(x, wx)
    /// We're preparing the value of g(x, wx) at all x here.
    /// Note that eq(w, wx, wx) = eq(w, wx)
    pub(crate) fn prepare_x_vals(&mut self) {
        let mul = &self.layer.mul;
        let add = &self.layer.add;
        let vals = &self.layer.input_vals;
        let eq_evals_at_rz0 = &mut self.sp.eq_evals_at_rz0;
        let eq_evals_at_rz1 = &mut self.sp.eq_evals_at_rz1;
        let eq_evals_at_rz_combined = &mut self.sp.eq_evals_at_rz_combined;
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

        let eq_evals_at_r_simd0 = C::Field::pack(&self.sp.eq_evals_at_r_simd0);
        let eq_evals_at_r_simd1 = C::Field::pack(&self.sp.eq_evals_at_r_simd1);

        for i in 0..1 << self.rz0.len() {
            eq_evals_at_rz_combined[i] =
                C::challenge_mul_field(&eq_evals_at_rz0[i], &eq_evals_at_r_simd0)
                    + C::challenge_mul_field(&eq_evals_at_rz1[i], &eq_evals_at_r_simd1);
        }

        for g in mul.iter() {
            let r = C::circuit_field_mul_simd_circuit_field(&g.coef, &vals[g.i_ids[1]]);
            hg_vals[g.i_ids[0]] +=
                C::field_mul_simd_circuit_field(&eq_evals_at_rz_combined[g.o_id], &r);

            gate_exists[g.i_ids[0]] = true;
        }

        for g in add.iter() {
            hg_vals[g.i_ids[0]] +=
                C::field_mul_circuit_field(&eq_evals_at_rz_combined[g.o_id], &g.coef);
            gate_exists[g.i_ids[0]] = true;
        }
    }

    /// We have V(rx, wx) g(rx, wx) at the end of x -> rx reduction
    /// Here we unpack simd value V(rx, wx) g(rx, wx) into vectors of length simd pack size
    /// and continue the reduction
    pub(crate) fn prepare_simd_var_x_vals(&mut self) {
        self.sp.simd_var_v_evals = self.sp.v_evals[0].unpack();
        self.sp.simd_var_hg_evals = self.sp.hg_evals[0].unpack();
    }

    /// \sum_{y, wy} V(rx, rwx) eq(rw, rwx, wy) mul(rz, rx, y) V(y, wy)
    /// Note: v_rx refers to the value V(rx, rwx)
    /// Note: eq(rw, rwx, wy) = eq(rw, wy) eq(rwx, wy) when wy are values on the boolean hypercube
    /// The above can be rewrote as
    /// V(rx, rwx) \sum_{y, wy} eq(rwx, wy) eq(rw, wy) mul(rz, rx, y) V(y, wy)
    pub(crate) fn prepare_y_vals(&mut self, v_rx: C::ChallengeField) {
        let mul = &self.layer.mul;
        let eq_evals_at_rz_combined = &mut self.sp.eq_evals_at_rz_combined;
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

        eq_eval_at(
            &self.r_simd_var_x,
            &C::ChallengeField::one(),
            &mut self.sp.eq_evals_at_r_simd0,
            &mut self.sp.eq_evals_first_half,
            &mut &mut self.sp.eq_evals_second_half,
        );
        let eq_evals_at_r_simdx = C::Field::pack(&self.sp.eq_evals_at_r_simd0);

        let prefix = C::challenge_mul_field(&v_rx, &eq_evals_at_r_simdx);
        for g in mul.iter() {
            hg_vals[g.i_ids[1]] += prefix
                * C::challenge_mul_field(
                    &C::challenge_mul_circuit_field(&eq_evals_at_rx[g.i_ids[0]], &g.coef),
                    &eq_evals_at_rz_combined[g.o_id],
                );
            gate_exists[g.i_ids[1]] = true;
        }
    }

    #[inline(always)]
    pub(crate) fn prepare_simd_var_y_vals(&mut self) {
        // same op because we do not distinguish the storage of x and y in this case
        self.prepare_simd_var_x_vals();
    }
}
