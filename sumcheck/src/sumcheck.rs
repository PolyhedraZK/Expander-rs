use circuit::CircuitLayer;
use config::{GKRConfig, MPIConfig};
use transcript::{Transcript, TranscriptInstance};

use crate::{
    sumcheck_helper::SumcheckGkrHelper, sumcheck_square_helper::SumcheckGkrSquareHelper,
    GkrScratchpad,
};

#[inline(always)]
fn transcript_io<C: GKRConfig>(
    ps: &[C::ChallengeField],
    transcript: &mut TranscriptInstance<C::FiatShamirHashType>,
    mpi_config: &MPIConfig,
) -> C::ChallengeField {
    assert!(ps.len() == 3 || ps.len() == 4); // 3 for x, y; 4 for simd var
    for p in ps {
        transcript.append_field_element::<C::ChallengeField>(p);
    }
    let mut r = transcript.generate_challenge::<C::ChallengeField>();
    mpi_config.root_broadcast(&mut r);
    r
}

// FIXME
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn sumcheck_prove_gkr_layer<C: GKRConfig>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    rz1: &Option<Vec<C::ChallengeField>>,
    r_simd: &[C::ChallengeField],
    r_mpi: &[C::ChallengeField],
    alpha: &C::ChallengeField,
    beta: &Option<C::ChallengeField>,
    transcript: &mut TranscriptInstance<C::FiatShamirHashType>,
    sp: &mut GkrScratchpad<C>,
    mpi_config: &MPIConfig,
) -> (
    Vec<C::ChallengeField>,
    Option<Vec<C::ChallengeField>>,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
) {
    let mut helper =
        SumcheckGkrHelper::new(layer, rz0, rz1, r_simd, r_mpi, alpha, beta, sp, mpi_config);

    helper.prepare_simd();
    helper.prepare_mpi();
    helper.prepare_x_vals();

    for i_var in 0..helper.input_var_num {
        let evals = helper.poly_evals_at_rx(i_var, 2);
        let r = transcript_io::<C>(&evals, transcript, mpi_config);
        helper.receive_rx(i_var, r);
    }

    helper.prepare_simd_var_vals();
    for i_var in 0..helper.simd_var_num {
        let evals = helper.poly_evals_at_r_simd_var(i_var, 3);
        let r = transcript_io::<C>(&evals, transcript, mpi_config);
        helper.receive_r_simd_var(i_var, r);
    }

    helper.prepare_mpi_var_vals();
    for i_var in 0..mpi_config.world_size().trailing_zeros() as usize {
        let evals = helper.poly_evals_at_r_mpi_var(i_var, 3);
        let r = transcript_io::<C>(&evals, transcript, mpi_config);
        helper.receive_r_mpi_var(i_var, r);
    }

    let vx_claim = helper.vx_claim();
    transcript.append_field_element::<C::ChallengeField>(&vx_claim);

    if !layer.structure_info.max_degree_one {
        helper.prepare_y_vals();
        for i_var in 0..helper.input_var_num {
            let evals = helper.poly_evals_at_ry(i_var, 2);
            let r = transcript_io::<C>(&evals, transcript, mpi_config);
            helper.receive_ry(i_var, r);
        }
        let vy_claim = helper.vy_claim();
        transcript.append_field_element::<C::ChallengeField>(&vy_claim);
    }

    let rx = helper.rx;
    let ry = if !layer.structure_info.max_degree_one {
        Some(helper.ry)
    } else {
        None
    };
    let r_simd = helper.r_simd_var;
    let r_mpi = helper.r_mpi_var;

    (rx, ry, r_simd, r_mpi)
}

// FIXME
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_range_loop)] // todo: remove
pub fn sumcheck_prove_gkr_square_layer<C: GKRConfig>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    transcript: &mut TranscriptInstance<C::FiatShamirHashType>,
    sp: &mut GkrScratchpad<C>,
) -> Vec<C::ChallengeField> {
    const D: usize = 7;
    let mut helper = SumcheckGkrSquareHelper::new(layer, rz0, sp);

    for i_var in 0..layer.input_var_num {
        if i_var == 0 {
            helper.prepare_g_x_vals();
        }
        let evals: [C::Field; D] = helper.poly_evals_at(i_var);

        for deg in 0..D {
            transcript.append_field_element::<C::Field>(&evals[deg]);
        }

        let r = transcript.generate_challenge::<C::ChallengeField>();

        log::trace!("i_var={} evals: {:?} r: {:?}", i_var, evals, r);

        helper.receive_challenge(i_var, r);
        if i_var == layer.input_var_num - 1 {
            log::trace!("vx claim: {:?}", helper.vx_claim());
            transcript.append_field_element::<C::Field>(&helper.vx_claim());
        }
    }

    log::trace!("claimed vx = {:?}", helper.vx_claim());
    transcript.append_field_element::<C::Field>(&helper.vx_claim());

    helper.rx
}

// #[cfg(test)]
// mod tests {
// use crate::BN254ConfigKeccak;

// type C = BN254ConfigKeccak;

// #[test]
// fn test_sumcheck_cuda() {
//     // Field: BN254 Scalar; Fiat Shamir Hash Function: Keccak256; Scheme: Vanilla GKR

//     // Sumcheck Outstanding Results
//     let mut rz0 = vec![];
//     let mut rz1 = vec![];

//     // Random Linear Combination
//     let mut alpha = <C as GKRConfig>::ChallengeField::one();
//     let mut beta = <C as GKRConfig>::ChallengeField::zero();

//     // Loading Circuit (hard-coded keccak circuit for now)
//     let mut circuit = Circuit::<BN254ConfigKeccak>::load_circuit(KECCAK_M31_CIRCUIT);
//     circuit.set_random_input_for_test();
//     circuit.evaluate();
//     let layer_num = circuit.layers.len();

//     // Define the scratchpad
//     let max_num_input_var = circuit
//         .layers
//         .iter()
//         .map(|layer| layer.input_var_num)
//         .max()
//         .unwrap();
//     let max_num_output_var = circuit
//         .layers
//         .iter()
//         .map(|layer| layer.output_var_num)
//         .max()
//         .unwrap();
//     let mut sp = GkrScratchpad::<BN254ConfigKeccak>::new(max_num_input_var, max_num_output_var);

//     // Do the PC commitment to initial the transcript
//     let commitment = RawCommitment::<C>::new(&circuit.layers[0].input_vals);
//     let mut buffer = vec![];
//     commitment.serialize_into(&mut buffer).unwrap(); // TODO: error propagation
//     let mut transcript = Transcript::<Keccak256hasher>::new();
//     transcript.append_u8_slice(&buffer);

//     // Do SumCheck Once from the output layer
//     (rz0, rz1) = sumcheck_prove_gkr_layer(
//         &circuit.layers[layer_num - 1],
//         &rz0,
//         &rz1,
//         &alpha,
//         &beta,
//         &mut transcript,
//         &mut sp,
//     );
//     alpha = transcript.challenge_f::<C>();
//     beta = transcript.challenge_f::<C>();
//     println!(
//         "rz0 = {:?}\nrz1 = {:?}\nalpha = {:?}, beta = {:?}",
//         rz0, rz1, alpha, beta
//     );
// }
// }
