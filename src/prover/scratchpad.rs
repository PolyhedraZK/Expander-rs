use arith::{Field, SimdField};

#[derive(Clone, Debug, Default)]
pub struct GkrScratchpad<F: Field + SimdField> {
    pub(crate) v_evals: Vec<F>,
    pub(crate) hg_evals_5: Vec<F>,
    pub(crate) hg_evals_1: Vec<F>,

    pub(crate) eq_evals_at_rx: Vec<F::Scalar>,
    pub(crate) eq_evals_at_rz0: Vec<F::Scalar>,
    pub(crate) eq_evals_at_rz1: Vec<F::Scalar>,
    pub(crate) eq_evals_first_half: Vec<F::Scalar>,
    pub(crate) eq_evals_second_half: Vec<F::Scalar>,

    pub(crate) gate_exists_5: Vec<bool>,
    pub(crate) gate_exists_1: Vec<bool>,
}

impl<F: Field + SimdField> GkrScratchpad<F> {
    pub(crate) fn new(max_num_input_var: usize, max_num_output_var: usize) -> Self {
        let max_input_num = 1 << max_num_input_var;
        let max_output_num = 1 << max_num_output_var;
        GkrScratchpad {
            v_evals: vec![F::default(); max_input_num],
            hg_evals_5: vec![F::default(); max_input_num],
            hg_evals_1: vec![F::default(); max_input_num],

            eq_evals_at_rx: vec![F::Scalar::default(); max_input_num],
            eq_evals_at_rz0: vec![F::Scalar::default(); max_output_num],
            eq_evals_at_rz1: vec![F::Scalar::default(); max_output_num],
            eq_evals_first_half: vec![F::Scalar::default(); max_output_num],
            eq_evals_second_half: vec![F::Scalar::default(); max_output_num],

            gate_exists_5: vec![false; max_input_num],
            gate_exists_1: vec![false; max_input_num],
        }
    }
}
