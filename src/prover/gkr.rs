//! This module implements the core GKR IOP.

use arith::{BinomialExtensionField, Field, FieldSerde, MultiLinearPoly, SimdField};
use ark_std::{end_timer, start_timer};

use crate::{sumcheck_prove_gkr_layer, Circuit, Config, GKRConfig, GkrScratchpad, Transcript};

// FIXME
#[allow(clippy::type_complexity)]
pub fn gkr_prove<C: GKRConfig>(
    circuit: &Circuit<C>,
    sp: &mut GkrScratchpad<C>,
    transcript: &mut Transcript,
    config: &Config,
) -> (C::Field, Vec<C::ChallengeField>, Vec<C::ChallengeField>) {
    let timer = start_timer!(|| "gkr prove");
    let layer_num = circuit.layers.len();

    let mut rz0 = vec![];
    let mut rz1 = vec![];
    for _i in 0..circuit.layers.last().unwrap().output_var_num {
        rz0.push(transcript.challenge_f::<C>());
        rz1.push(C::ChallengeField::zero());
    }

    let mut alpha = C::ChallengeField::one();
    let mut beta = C::ChallengeField::zero();

    let claimed_v = MultiLinearPoly::<C::Field>::eval_multilinear(
        &circuit.layers.last().unwrap().output_vals.evals,
        &rz0,
    );

    for i in (0..layer_num).rev() {
        (rz0, rz1) = sumcheck_prove_gkr_layer(
            &circuit.layers[i],
            &rz0,
            &rz1,
            &alpha,
            &beta,
            transcript,
            sp,
            config,
        );
        alpha = transcript.challenge_f::<C>();
        beta = transcript.challenge_f::<C>();

        log::trace!("Layer {} proved with alpha={:?}, beta={:?}", i, alpha, beta);
        log::trace!("rz0.0: {:?}", rz0[0]);
        log::trace!("rz0.1: {:?}", rz0[1]);
        log::trace!("rz0.2: {:?}", rz0[2]);
        log::trace!("rz1.0: {:?}", rz1[0]);
        log::trace!("rz1.1: {:?}", rz1[1]);
        log::trace!("rz1.2: {:?}", rz1[2]);
    }

    end_timer!(timer);
    (claimed_v, rz0, rz1)
}
