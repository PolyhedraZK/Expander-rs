mod common;

use arith::{Field, Fr};
use ark_std::test_rng;
use gkr_engine::ExpanderPCS;
use gkr_engine::{BN254Config, ExpanderSingleVarChallenge, MPIConfig, MPIEngine, StructuredReferenceString, Transcript};
use halo2curves::group::prime::PrimeCurveAffine;
use halo2curves::group::Curve;
use poly_commit::PolynomialCommitmentScheme;
use gkr_hashers::Keccak256hasher;
use halo2curves::bn256::Bn256;
use poly_commit::HyperUniKZGPCS;
use polynomials::MultiLinearPoly;
use transcript::BytesHashTranscript;

const TEST_REPETITION: usize = 3;

fn test_hyperkzg_pcs_generics(num_vars_start: usize, num_vars_end: usize) {
    let mut rng = test_rng();

    (num_vars_start..=num_vars_end).for_each(|num_vars| {
        let xs: Vec<_> = (0..TEST_REPETITION)
            .map(|_| -> Vec<Fr> { (0..num_vars).map(|_| Fr::random_unsafe(&mut rng)).collect() })
            .collect();
        let poly = MultiLinearPoly::<Fr>::random(num_vars, &mut rng);

        common::test_pcs::<Fr, BytesHashTranscript<Keccak256hasher>, HyperUniKZGPCS<Bn256>>(
            &num_vars, &poly, &xs,
        );
    })
}

#[test]
fn test_hyper_uni_kzg_pcs_full_e2e() {
    test_hyperkzg_pcs_generics(2, 15)
}

fn test_hyper_unikzg_for_expander_gkr_generics(mpi_config_ref: &MPIConfig, total_num_vars: usize) {
    let mut rng = test_rng();

    // NOTE BN254 GKR SIMD pack size = 1, num vars in SIMD is 0
    let num_vars_in_mpi = mpi_config_ref.world_size().ilog2() as usize;
    let num_vars_in_each_poly = total_num_vars - num_vars_in_mpi;

    let global_poly = MultiLinearPoly::<Fr>::random(total_num_vars, &mut rng);
    let challenge_point = ExpanderSingleVarChallenge::<BN254Config> {
        r_mpi: (0..num_vars_in_mpi)
            .map(|_| Fr::random_unsafe(&mut rng))
            .collect(),
        r_simd: Vec::new(),
        rz: (0..num_vars_in_each_poly)
            .map(|_| Fr::random_unsafe(&mut rng))
            .collect(),
    };

    let mut transcript = BytesHashTranscript::<Keccak256hasher>::new();

    // NOTE separate polynomial into different pieces by mpi rank
    let poly_vars_stride = (1 << global_poly.get_num_vars()) / mpi_config_ref.world_size();
    let poly_coeff_starts = mpi_config_ref.world_rank() * poly_vars_stride;
    let poly_coeff_ends = poly_coeff_starts + poly_vars_stride;
    let local_poly =
        MultiLinearPoly::new(global_poly.coeffs[poly_coeff_starts..poly_coeff_ends].to_vec());

    dbg!(local_poly.get_num_vars(), local_poly.coeffs[0]);

    let params = <HyperUniKZGPCS<Bn256> as ExpanderPCS<BN254Config>>::gen_params(
        num_vars_in_each_poly,
        mpi_config_ref.world_size(),
    );
    common::test_pcs_for_expander_gkr::<
        BN254Config,
        BytesHashTranscript<Keccak256hasher>,
        HyperUniKZGPCS<Bn256>,
    >(
        &params,
        mpi_config_ref,
        &mut transcript,
        &local_poly,
        &[challenge_point],
        None,
    );
}

#[test]
fn test_hyper_unikzg_for_expander_gkr() {
    let universe = MPIConfig::init().unwrap();
    let world = universe.world();
    let mpi_config = MPIConfig::prover_new(Some(&universe), Some(&world));

    test_hyper_unikzg_for_expander_gkr_generics(&mpi_config, 0);
    test_hyper_unikzg_for_expander_gkr_generics(&mpi_config, 1);
    test_hyper_unikzg_for_expander_gkr_generics(&mpi_config, 15);
}

#[test]
fn test_uni_kzg_batch_open() {
    common::test_batching::<Fr, BytesHashTranscript<Keccak256hasher>, HyperUniKZGPCS<Bn256>>();
    common::test_batching_for_expander_gkr::<
        BN254Config,
        BytesHashTranscript<Keccak256hasher>,
        HyperUniKZGPCS<Bn256>,
    >(true);
}

#[test]
fn test_hyperkzg_rejects_corrupted_proof() {
    let mut rng = test_rng();
    let num_vars = 4;

    let (srs, _) =
        <HyperUniKZGPCS<Bn256> as PolynomialCommitmentScheme<Fr>>::gen_srs_for_testing(
            &num_vars, &mut rng,
        );
    let (proving_key, verification_key) = srs.into_keys();

    let poly = MultiLinearPoly::<Fr>::random(num_vars, &mut rng);
    let x: Vec<Fr> = (0..num_vars).map(|_| Fr::random_unsafe(&mut rng)).collect();

    let mut scratch_pad = ();
    let commitment =
        <HyperUniKZGPCS<Bn256> as PolynomialCommitmentScheme<Fr>>::commit(
            &num_vars, &proving_key, &poly, &mut scratch_pad,
        );

    let mut transcript = BytesHashTranscript::<Keccak256hasher>::new();
    let (eval, opening) = <HyperUniKZGPCS<Bn256> as PolynomialCommitmentScheme<Fr>>::open(
        &num_vars, &proving_key, &poly, &x, &scratch_pad, &mut transcript,
    );

    // Valid proof should pass
    let mut transcript_v = BytesHashTranscript::<Keccak256hasher>::new();
    assert!(<HyperUniKZGPCS<Bn256> as PolynomialCommitmentScheme<Fr>>::verify(
        &num_vars, &verification_key, &commitment, &x, eval, &opening, &mut transcript_v,
    ));

    // Corrupted proof should fail
    let mut corrupted = opening.clone();
    corrupted.quotient_delta_x_commitment =
        (corrupted.quotient_delta_x_commitment.to_curve() * Fr::from(2u64)).to_affine();

    let mut transcript_c = BytesHashTranscript::<Keccak256hasher>::new();
    assert!(!<HyperUniKZGPCS<Bn256> as PolynomialCommitmentScheme<Fr>>::verify(
        &num_vars, &verification_key, &commitment, &x, eval, &corrupted, &mut transcript_c,
    ));
}
