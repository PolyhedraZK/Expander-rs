use arith::SimdField;

// #[inline(always)]
// pub(crate) fn unpack_and_sum<F: SimdField>(p: &F) -> F::Scalar {
//     p.unpack().into_iter().sum()
// }

#[inline(always)]
/// Input
/// - a SIMD field, denoted by p := [p0, ... pn]
/// - a vector of coefficients, denoted by coef := [c0, ... cn]
///
/// Output
/// - p0 * c0 + ... + pn * cn
pub fn unpack_and_combine<F: SimdField>(p: &F, coef: &[F::Scalar]) -> F::Scalar {
    let p_unpacked = p.unpack();
    p_unpacked
        .into_iter()
        .zip(coef)
        .map(|(p_i, coef_i)| p_i * coef_i)
        .sum()
}
