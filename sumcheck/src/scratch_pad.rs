use config::GKRConfig;

#[derive(Clone, Debug, Default)]
pub struct GkrScratchpad<C: GKRConfig> {
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

impl<C: GKRConfig> GkrScratchpad<C> {
    pub fn new(max_num_input_var: usize, max_num_output_var: usize, mpi_world_size: usize) -> Self {
        let max_input_num = 1 << max_num_input_var;
        let max_output_num = 1 << max_num_output_var;
        GkrScratchpad {
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
