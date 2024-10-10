//! This module implements helper functions for the prover side of the sumcheck protocol
//! to evaluate SIMD gates. A SIMD gate is a gate that merges all elements in a SIMD into a single one.

use arith::{ExtensionField, Field};
use config::{FieldType, GKRConfig};

pub(crate) struct SumcheckSimdProdGateHelper {
    var_num: usize,
}

// The logic is exactly the same as SumcheckProductGateHelper, but field types are different
impl SumcheckSimdProdGateHelper {
    pub(crate) fn new(var_num: usize) -> Self {
        SumcheckSimdProdGateHelper { var_num }
    }

    #[inline]
    pub(crate) fn poly_eval_at<C: GKRConfig>(
        &self,
        var_idx: usize,
        degree: usize,
        bk_eq: &mut [C::ChallengeField],
        bk_f: &mut [C::ChallengeField],
        bk_hg: &mut [C::ChallengeField],
    ) -> [C::ChallengeField; 4] {
        assert_eq!(degree, 3);
        let mut p0 = C::ChallengeField::zero();
        let mut p1 = C::ChallengeField::zero();
        let mut p2 = C::ChallengeField::zero();
        let mut p3 = C::ChallengeField::zero();

        let eval_size = 1 << (self.var_num - var_idx - 1);

        if C::FIELD_TYPE == FieldType::GF2 {
            for i in 0..eval_size {
                let eq_v_0 = bk_eq[i * 2];
                let eq_v_1 = bk_eq[i * 2 + 1];
                let f_v_0 = bk_f[i * 2];
                let f_v_1 = bk_f[i * 2 + 1];
                let hg_v_0 = bk_hg[i * 2];
                let hg_v_1 = bk_hg[i * 2 + 1];

                p0 += eq_v_0 * f_v_0 * hg_v_0;
                p1 += eq_v_1 * f_v_1 * hg_v_1;

                let eq_linear = (eq_v_1 - eq_v_0).mul_by_x();
                let f_linear = (f_v_1 - f_v_0).mul_by_x();
                let hg_linear = (hg_v_1 - hg_v_0).mul_by_x();

                // evaluated at x and x^2 for p2 and p3
                p2 += (eq_linear + eq_v_0) * (f_linear + f_v_0) * (hg_linear + hg_v_0);
                p3 += (eq_linear.mul_by_x() + eq_v_0)
                    * (f_linear.mul_by_x() + f_v_0)
                    * (hg_linear.mul_by_x() + hg_v_0);
            }
        } else {
            for i in 0..eval_size {
                let eq_v_0 = bk_eq[i * 2];
                let eq_v_1 = bk_eq[i * 2 + 1];
                let f_v_0 = bk_f[i * 2];
                let f_v_1 = bk_f[i * 2 + 1];
                let hg_v_0 = bk_hg[i * 2];
                let hg_v_1 = bk_hg[i * 2 + 1];

                p0 += eq_v_0 * f_v_0 * hg_v_0;
                p1 += eq_v_1 * f_v_1 * hg_v_1;

                // evaluated at 2 and 3 for p2 and p3
                let tmp0 = eq_v_1 - eq_v_0;
                let tmp1 = f_v_1 - f_v_0;
                let tmp2 = hg_v_1 - hg_v_0;
                let tmp3 = eq_v_1 + tmp0;
                let tmp4 = f_v_1 + tmp1;
                let tmp5 = hg_v_1 + tmp2;

                p2 += tmp3 * tmp4 * tmp5;
                p3 += (tmp3 + tmp0) * (tmp4 + tmp1) * (tmp5 + tmp2);
            }
        }

        [p0, p1, p2, p3]
    }

    #[inline]
    pub(crate) fn receive_challenge<C: GKRConfig>(
        &mut self,
        var_idx: usize,
        r: C::ChallengeField,
        bk_eq: &mut [C::ChallengeField],
        bk_f: &mut [C::ChallengeField],
        bk_hg: &mut [C::ChallengeField],
    ) {
        assert!(var_idx < self.var_num);

        let eval_size = 1 << (self.var_num - var_idx - 1);
        for i in 0..eval_size {
            bk_eq[i] = bk_eq[2 * i] + (bk_eq[2 * i + 1] - bk_eq[2 * i]) * r;
            bk_f[i] = bk_f[2 * i] + (bk_f[2 * i + 1] - bk_f[2 * i]) * r;
            bk_hg[i] = bk_hg[2 * i] + (bk_hg[2 * i + 1] - bk_hg[2 * i]) * r;
        }
    }
}
