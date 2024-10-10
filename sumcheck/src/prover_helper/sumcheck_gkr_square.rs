use arith::Field;
use circuit::CircuitLayer;
use config::GKRConfig;
use polynomials::EqPolynomial;

use crate::ProverScratchPad;

use super::power_gate::SumcheckPowerGateHelper;

// todo: Move D to GKRConfig
pub(crate) struct SumcheckGkrSquareHelper<'a, C: GKRConfig, const D: usize> {
    pub(crate) rx: Vec<C::ChallengeField>,

    layer: &'a CircuitLayer<C>,
    sp: &'a mut ProverScratchPad<C>,
    rz0: &'a [C::ChallengeField],

    _input_var_num: usize,
    _output_var_num: usize,

    x_helper: SumcheckPowerGateHelper<D>,
}

impl<'a, C: GKRConfig, const D: usize> SumcheckGkrSquareHelper<'a, C, D> {
    #[inline]
    pub(crate) fn new(
        layer: &'a CircuitLayer<C>,
        rz0: &'a [C::ChallengeField],
        sp: &'a mut ProverScratchPad<C>,
    ) -> Self {
        SumcheckGkrSquareHelper {
            rx: vec![],

            layer,
            sp,
            rz0,

            _input_var_num: layer.input_var_num,
            _output_var_num: layer.output_var_num,

            x_helper: SumcheckPowerGateHelper::new(layer.input_var_num),
        }
    }

    #[inline]
    pub(crate) fn poly_evals_at(&self, var_idx: usize) -> [C::Field; D] {
        self.x_helper.poly_eval_at::<C>(
            var_idx,
            &self.sp.v_evals,
            &self.sp.hg_evals_5,
            &self.sp.hg_evals_1,
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

    #[inline]
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
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
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
