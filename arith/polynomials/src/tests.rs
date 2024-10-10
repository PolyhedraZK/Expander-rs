use super::*;
use arith::Field;
use ark_std::test_rng;
use halo2curves::bn256::Fr;

#[test]
fn test_scaled_eq_xr() {
    let mut rng = test_rng();
    for nv in 4..10 {
        let r: Vec<Fr> = (0..nv).map(|_| Fr::random_unsafe(&mut rng)).collect();
        let scalar = Fr::random_unsafe(&mut rng);

        // expander
        let mut eq_x_r1 = vec![Fr::zero(); 1 << nv];
        EqPolynomial::<Fr>::build_eq_x_r_with_buf(r.as_ref(), &scalar, &mut eq_x_r1);

        // jolt
        let eq_x_r2 = EqPolynomial::<Fr>::scaled_evals_jolt(r.as_ref(), &scalar);

        assert_eq!(eq_x_r1, eq_x_r2);
    }
}

#[test]
fn test_mle_eval() {
    let mut rng = test_rng();
    for nv in 4..10 {
        let mle = MultiLinearPoly::<Fr>::random(nv, &mut rng);
        let point = (0..nv)
            .map(|_| Fr::random_unsafe(&mut rng))
            .collect::<Vec<_>>();

        // jolt's method
        let eval_via_eq = mle.evaluate_jolt(&point);

        // hyperplonk's method
        let mut mle_eval = mle.clone();
        mle_eval.fix_variables(point.as_ref());
        assert!(mle_eval.coeffs.len() == 1);
        assert_eq!(eval_via_eq, mle_eval.coeffs[0]);

        // expander's method
        let mut buf = vec![Fr::zero(); 1 << nv];
        MultiLinearPoly::<Fr>::evaluate_with_buffer(
            mle.coeffs.as_ref(),
            point.as_ref(),
            buf.as_mut(),
        );
        assert_eq!(eval_via_eq, buf[0]);
    }
}

#[test]
fn test_eq_xr() {
    let mut rng = test_rng();
    for nv in 4..10 {
        let r: Vec<Fr> = (0..nv).map(|_| Fr::random_unsafe(&mut rng)).collect();

        // naive
        let eq_x_r0 = build_eq_x_r_for_test(r.as_ref());

        // hyperplonk
        let eq_x_r1 = EqPolynomial::<Fr>::build_eq_x_r(r.as_ref());
        assert_eq!(eq_x_r1, eq_x_r0);

        // expander
        let mut eq_x_r2 = vec![Fr::zero(); 1 << nv];
        EqPolynomial::<Fr>::build_eq_x_r_with_buf(r.as_ref(), &Fr::ONE, &mut eq_x_r2);
        assert_eq!(eq_x_r2, eq_x_r0);

        // jolt
        let eq_x_r3 = EqPolynomial::<Fr>::evals_jolt(r.as_ref());
        assert_eq!(eq_x_r3, eq_x_r0);
    }
}

/// Naive method to build eq(x, r).
/// Only used for testing purpose.
// Evaluate
//      eq(x,y) = \prod_i=1^num_var (x_i * y_i + (1-x_i)*(1-y_i))
// over r, which is
//      eq(x,y) = \prod_i=1^num_var (x_i * r_i + (1-x_i)*(1-r_i))
fn build_eq_x_r_for_test<F: Field>(r: &[F]) -> Vec<F> {
    // we build eq(x,r) from its evaluations
    // we want to evaluate eq(x,r) over x \in {0, 1}^num_vars
    // for example, with num_vars = 4, x is a binary vector of 4, then
    //  0 0 0 0 -> (1-r0)   * (1-r1)    * (1-r2)    * (1-r3)
    //  1 0 0 0 -> r0       * (1-r1)    * (1-r2)    * (1-r3)
    //  0 1 0 0 -> (1-r0)   * r1        * (1-r2)    * (1-r3)
    //  1 1 0 0 -> r0       * r1        * (1-r2)    * (1-r3)
    //  ....
    //  1 1 1 1 -> r0       * r1        * r2        * r3
    // we will need 2^num_var evaluations

    // First, we build array for {1 - r_i}
    let one_minus_r: Vec<F> = r.iter().map(|ri| F::one() - ri).collect();

    let num_var = r.len();
    let mut eval = vec![];

    for i in 0..1 << num_var {
        let mut current_eval = F::one();
        let bit_sequence = bit_decompose(i, num_var);

        for (&bit, (ri, one_minus_ri)) in bit_sequence.iter().zip(r.iter().zip(one_minus_r.iter()))
        {
            current_eval *= if bit { *ri } else { *one_minus_ri };
        }
        eval.push(current_eval);
    }

    eval
}

/// Decompose an integer into a binary vector in little endian.
fn bit_decompose(input: u64, num_var: usize) -> Vec<bool> {
    let mut res = Vec::with_capacity(num_var);
    let mut i = input;
    for _ in 0..num_var {
        res.push(i & 1 == 1);
        i >>= 1;
    }
    res
}
