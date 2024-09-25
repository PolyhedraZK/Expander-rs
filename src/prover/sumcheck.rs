use crate::{
    CircuitLayer, GKRConfig, GkrScratchpad, SumcheckGkrHelper, SumcheckGkrSquareHelper, Transcript,
};

#[inline(always)]
fn transcript_io<C: GKRConfig>(
    ps: &[C::ChallengeField],
    transcript: &mut Transcript<C::FiatShamirHashType>,
) -> C::ChallengeField {
    assert!(ps.len() == 3 || ps.len() == 4); // 3 for x, y; 4 for simd var
    for p in ps {
        transcript.append_challenge_f::<C>(p);
    }
    transcript.challenge_f::<C>()
}

// FIXME
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn sumcheck_prove_gkr_layer<C: GKRConfig>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    rz1: &[C::ChallengeField],
    r_simd: &[C::ChallengeField],
    alpha: &C::ChallengeField,
    beta: &C::ChallengeField,
    transcript: &mut Transcript<C::FiatShamirHashType>,
    sp: &mut GkrScratchpad<C>,
) -> (
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
) {
    let mut helper = SumcheckGkrHelper::new(layer, rz0, rz1, r_simd, alpha, beta, sp);

    helper.prepare_simd();
    helper.prepare_x_vals();

    for i_var in 0..helper.input_var_num {
        let evals = helper.poly_evals_at_rx(i_var, 2);
        let r = transcript_io::<C>(&evals, transcript);
        helper.receive_rx(i_var, r);
    }

    helper.prepare_simd_var_vals();
    for i_var in 0..helper.simd_var_num {
        let evals = helper.poly_evals_at_r_simd_var(i_var, 3);
        let r = transcript_io::<C>(&evals, transcript);
        helper.receive_r_simd_var(i_var, r);
    }

    let vx_claim = helper.vx_claim();
    transcript.append_challenge_f::<C>(&vx_claim);
    helper.prepare_y_vals();
    for i_var in 0..helper.input_var_num {
        let evals = helper.poly_evals_at_ry(i_var, 2);
        let r = transcript_io::<C>(&evals, transcript);
        helper.receive_ry(i_var, r);
    }

    let vy_claim = helper.vy_claim();
    transcript.append_challenge_f::<C>(&vy_claim);

    let rx = helper.rx;
    let ry = helper.ry;
    let r_simd = helper.r_simd_var;

    (rx, ry, r_simd)
}

// FIXME
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_range_loop)] // todo: remove
pub fn sumcheck_prove_gkr_square_layer<C: GKRConfig>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    transcript: &mut Transcript<C::FiatShamirHashType>,
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
            transcript.append_f::<C>(evals[deg]);
        }

        let r = transcript.challenge_f::<C>();

        log::trace!("i_var={} evals: {:?} r: {:?}", i_var, evals, r);

        helper.receive_challenge(i_var, r);
        if i_var == layer.input_var_num - 1 {
            log::trace!("vx claim: {:?}", helper.vx_claim());
            transcript.append_f::<C>(helper.vx_claim());
        }
    }

    log::trace!("claimed vx = {:?}", helper.vx_claim());
    transcript.append_f::<C>(helper.vx_claim());

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
