//! Definition of polynomial commitment scheme
// TODO: merge with bi-kzg/src/pcs.rs

use std::{borrow::Borrow, fmt::Debug};

// use arith::Field;
use rand::RngCore;

/// This trait defines APIs for polynomial commitment schemes.
/// Note that for our usage of PCS, we do not require the hiding property.
///
/// Credit: https://github.com/EspressoSystems/hyperplonk/blob/8698369edfe82bd6617a9609602380f21cabd1da/subroutines/src/pcs/mod.rs#L24
pub trait PolynomialCommitmentScheme {
    /// Prover parameters
    type ProverParam: Clone;
    /// Verifier parameters
    type VerifierParam: Clone;
    /// Structured reference string
    type SRS: Clone + Debug;
    /// Polynomial and its associated types
    type Polynomial: Clone + Debug;
    /// Polynomial input domain
    type Point: Clone + Debug + PartialEq;
    /// Polynomial Evaluation
    type Evaluation;
    /// Commitments
    type Commitment: Clone + Debug;
    /// Proofs
    type Proof: Clone + Debug;
    /// Batch proofs
    type BatchProof;
    /// Transcript
    type Transcript: Clone;

    /// Build SRS for testing.
    ///
    ///
    /// WARNING: THIS FUNCTION IS FOR TESTING PURPOSE ONLY.
    /// THE OUTPUT SRS SHOULD NOT BE USED IN PRODUCTION.
    fn gen_srs_for_testing(rng: impl RngCore, supported_n: usize, supported_m: usize) -> Self::SRS;

    /// Generate a commitment for a polynomial
    /// ## Note on function signature
    /// Usually, data structure like SRS and ProverParam are huge and users
    /// might wish to keep them in heap using different kinds of smart pointers
    /// (instead of only in stack) therefore our `impl Borrow<_>` interface
    /// allows for passing in any pointer type, e.g.: `commit(prover_param:
    /// &Self::ProverParam, ..)` or `commit(prover_param:
    /// Box<Self::ProverParam>, ..)` or `commit(prover_param:
    /// Arc<Self::ProverParam>, ..)` etc.
    fn commit(
        prover_param: impl Borrow<Self::ProverParam>,
        polynomial: &Self::Polynomial,
    ) -> Self::Commitment;

    /// On input a polynomial `p` and a point `point`, outputs a proof for the
    /// same.
    fn open(
        prover_param: impl Borrow<Self::ProverParam>,
        commitment: &Self::Commitment,
        polynomial: &Self::Polynomial,
        opening_point: &Self::Point,
        transcript: &mut Self::Transcript,
    ) -> Self::Proof;

    /// Input a list of polynomials, and a same number of points, compute a multi-opening for all the polynomials.
    fn multi_open(
        _prover_param: impl Borrow<Self::ProverParam>,
        _polynomials: &[Self::Polynomial],
        _points: &[Self::Point],
        _evals: &[Self::Evaluation],
    ) -> Self::BatchProof {
        // the reason we use unimplemented!() is to enable developers to implement the
        // trait without always implementing the batching APIs.
        unimplemented!()
    }

    /// Verifies that `value` is the evaluation at `x` of the polynomial
    /// committed inside `comm`.
    fn verify(
        verifier_param: &Self::VerifierParam,
        commitment: &Self::Commitment,
        point: &Self::Point,
        value: &Self::Evaluation,
        proof: &Self::Proof,
    ) -> bool;

    /// Verifies that `value_i` is the evaluation at `x_i` of the polynomial
    /// `poly_i` committed inside `comm`.
    fn batch_verify(
        _verifier_param: &Self::VerifierParam,
        _commitments: &[Self::Commitment],
        _points: &[Self::Point],
        _batch_proof: &Self::BatchProof,
    ) -> bool {
        // the reason we use unimplemented!() is to enable developers to implement the
        // trait without always implementing the batching APIs.
        unimplemented!()
    }
}
