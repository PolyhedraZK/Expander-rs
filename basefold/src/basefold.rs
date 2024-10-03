use std::ops::Mul;

use arith::{ExtensionField, FFTField, FieldSerde};
use ark_std::{end_timer, start_timer};
use babybear::BabyBearx16;
use mpoly::MultiLinearPoly;
// use p3_baby_bear::PackedBabyBearAVX512 as BabyBearx16;
use rand::RngCore;
use transcript::{FiatShamirHash, Transcript};
use tree::Tree;

use crate::{
    iop::BasefoldIOPPQuery, BasefoldCommitment, BasefoldIOPPQuerySingleRound, BasefoldParam,
    BasefoldProof, PolynomialCommitmentScheme, LOG_RATE,
};

pub struct BaseFoldPCS<T, H, ExtF, F> {
    pub transcript: std::marker::PhantomData<T>,
    pub hasher: std::marker::PhantomData<H>,
    pub field: std::marker::PhantomData<F>,
    pub ext_field: std::marker::PhantomData<ExtF>,
}

impl<T, H, ExtF, F> PolynomialCommitmentScheme for BaseFoldPCS<T, H, ExtF, F>
where
    T: Transcript<H>,
    H: FiatShamirHash,
    F: FFTField + FieldSerde,
    ExtF: ExtensionField<BaseField = F>,
{
    type ProverParam = BasefoldParam<T, H, ExtF, F>;
    type VerifierParam = BasefoldParam<T, H, ExtF, F>;
    type SRS = BasefoldParam<T, H, ExtF, F>;
    type Polynomial = MultiLinearPoly<F>;
    type Point = Vec<F>;
    type Evaluation = ExtF;
    type Commitment = BasefoldCommitment<F>;
    type Proof = BasefoldProof<ExtF>;
    type BatchProof = ();
    type Transcript = T;

    fn gen_srs_for_testing(
        _rng: impl RngCore,
        _supported_n: usize,
        _supported_m: usize,
    ) -> Self::SRS {
        BasefoldParam::<T, H, ExtF, F>::new(LOG_RATE)
    }

    fn commit(
        prover_param: impl std::borrow::Borrow<Self::ProverParam>,
        polynomial: &Self::Polynomial,
    ) -> Self::Commitment {
        let timer = start_timer!(|| format!(
            "basefold commit poly with {} vars",
            polynomial.get_num_vars()
        ));
        let commit = BasefoldCommitment {
            tree: prover_param.borrow().basefold_oracle_from_poly(polynomial),
        };
        end_timer!(timer);

        commit
    }

    fn open(
        prover_param: impl std::borrow::Borrow<Self::ProverParam>,
        commitment: &Self::Commitment,
        polynomial: &Self::Polynomial,
        opening_point: &Self::Point,
        transcript: &mut T,
    ) -> Self::Proof {
        let timer = start_timer!(|| format!(
            "basefold prove poly with {} vars",
            polynomial.get_num_vars()
        ));

        let shift_z =
            EqPolynomial::new(opening_point.iter().cloned().map(Into::into).collect_vec())
                .to_dense();
        let mut sumcheck_poly_vec = vec![polynomial.lift_to::<ExtF>(), shift_z];
        let merge_function = |x: &[ExtF]| x.iter().product::<ExtF>();

        let num_vars = polynomial.get_num_vars();

        let mut sumcheck_polys: Vec<_> = Vec::with_capacity(num_vars);
        let mut iopp_codewords: Vec<_> = Vec::with_capacity(num_vars);

        let hasher = H::new_instance();

        (0..num_vars).for_each(|_| {
            // NOTE: sumcheck a single step, r_i start from x_0 towards x_n
            let (sc_univariate_poly_i, _, _) = SumcheckInstanceProof::prove_arbitrary(
                &ExtF::zero(),
                1,
                &mut sumcheck_poly_vec,
                merge_function,
                MERGE_POLY_DEG,
                transcript,
            );
            sumcheck_polys.push(sc_univariate_poly_i.compressed_polys[0].clone());
            drop(sc_univariate_poly_i);

            let coeffs = sumcheck_poly_vec[0].interpolate_over_hypercube();
            iopp_codewords.push(prover_param.borrow().reed_solomon_from_coeffs(coeffs));
        });

        let iopp_oracles = Tree::batch_tree_for_recursive_oracles(iopp_codewords, &hasher);

        let iopp_last_oracle_message = iopp_oracles[iopp_oracles.len() - 1].leaves.clone();
        let iopp_challenges = prover_param.borrow().iopp_challenges(num_vars, transcript);

        let iopp_queries = (0..prover_param.borrow().verifier_queries)
            .zip(iopp_challenges)
            .map(|(_, mut point)| {
                let mut iopp_round_query = Vec::with_capacity(iopp_oracles.len() + 1);

                // Merkle queries over F
                let oracle_rhs_start = commitment.tree.size() >> 1;
                let sibling_point = point ^ oracle_rhs_start;
                let left = std::cmp::min(point, sibling_point);
                let right = oracle_rhs_start + left;
                point = left;

                iopp_round_query.push(BasefoldIOPPQuerySingleRound {
                    left: commitment.tree.index_query(left),
                    right: commitment.tree.index_query(right),
                });

                // Merkle queries over ExtF
                iopp_oracles.iter().for_each(|oracle| {
                    // NOTE: since the oracle length is always a power of 2,
                    // so the right hand side starts from directly div by 2.
                    let oracle_rhs_start = oracle.size() >> 1;

                    // NOTE: dirty trick, oracle rhs starting index is a pow of 2.
                    // now that we want to find a sibling point w.r.t the index,
                    // by plus (or minus) against point, so xor should work.
                    let sibling_point = point ^ oracle_rhs_start;

                    let left = std::cmp::min(point, sibling_point);
                    let right = oracle_rhs_start + left;

                    // NOTE: update point for next round of IOPP querying
                    point = left;

                    iopp_round_query.push(BasefoldIOPPQuerySingleRound {
                        left: oracle.index_query(left),
                        right: oracle.index_query(right),
                    })
                });

                BasefoldIOPPQuery { iopp_round_query }
            })
            .collect();
        end_timer!(timer);

        BasefoldProof {
            // sumcheck_transcript: SumcheckInstanceProof::new(sumcheck_polys),
            iopp_oracles: iopp_oracles.iter().map(|t| t.root()).collect(),
            iopp_last_oracle_message,
            iopp_queries,
        }
    }

    fn verify(
        _verifier_param: &Self::VerifierParam,
        _commitment: &Self::Commitment,
        _point: &Self::Point,
        _value: &Self::Evaluation,
        _proof: &Self::Proof,
    ) -> bool {
        unimplemented!()
    }
}
