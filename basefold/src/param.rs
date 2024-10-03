use arith::{ExtensionField, FFTField, Field, FieldSerde};
// use arith::{FFTField, Field};
use ark_std::{end_timer, start_timer};
use mpoly::MultiLinearPoly;
// use p3_dft::TwoAdicSubgroupDft;
// use p3_field::{ExtensionField, TwoAdicField};
// use p3_matrix::{dense::RowMajorMatrix, Matrix};
// use rayon::iter::{
//     IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
// };
use transcript::{FiatShamirHash, Transcript};
use tree::Tree;

// use crate::Babybearx16

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BasefoldParam<T, H, ExtF, F>
where
    T: Transcript<H>,
    H: FiatShamirHash,
    F: FFTField + FieldSerde,
    ExtF: ExtensionField<BaseField = F>,
{
    // log(rho) where rho is the rate of the code
    pub rate_bits: usize,
    // number of queries
    pub verifier_queries: usize,

    pub transcript: std::marker::PhantomData<T>,
    pub hasher: std::marker::PhantomData<H>,
    pub field: std::marker::PhantomData<F>,
    pub ext_field: std::marker::PhantomData<ExtF>,
}

impl<T, H, ExtF, F> BasefoldParam<T, H, ExtF, F>
where
    T: Transcript<H>,
    H: FiatShamirHash,
    F: FFTField + FieldSerde,
    ExtF: ExtensionField<BaseField = F>,
{
    pub fn new(rate_bits: usize) -> Self {
        // TODO: this number is arbitrary, need further analysis.
        let verifier_queries = 80;

        Self {
            rate_bits,
            verifier_queries,
            transcript: std::marker::PhantomData,
            hasher: std::marker::PhantomData,
            field: std::marker::PhantomData,
            ext_field: std::marker::PhantomData,
        }
    }

    #[inline]
    /// Generate a list of positions that we want to open the polynomial at.
    pub fn iopp_challenges(&self, num_vars: usize, transcript: &mut T) -> Vec<usize> {
        let iopp_challenge_bitmask = (1 << self.codeword_bits(num_vars)) - 1;

        // NOTE: Fiat-Shamir sampling an IOPP query point ranging [0, 2^codeword_bits - 1].
        transcript
            .generate_challenge_index_vector(self.verifier_queries)
            .iter()
            .map(|c| c & iopp_challenge_bitmask)
            .collect()
    }

    #[inline]
    pub fn codeword_bits(&self, num_vars: usize) -> usize {
        self.rate_bits + num_vars
    }

    #[inline]
    pub fn t_term(&self, num_vars: usize, round: usize, index: usize) -> F {
        let round_gen = F::two_adic_generator(self.codeword_bits(num_vars) - round);
        round_gen.exp(index as u128)
    }

    #[inline]
    pub fn reed_solomon_from_coeffs(&self, mut coeffs: Vec<F>) -> Vec<F> {
        plonky2_util::reverse_index_bits_in_place(&mut coeffs);
        let extended_length = coeffs.len() << self.rate_bits;
        coeffs.resize(extended_length, F::zero());
        F::fft(&coeffs)
    }

    /// Performs dft in batch. returns a vector that is concatenated from all the dft results.
    fn batch_reed_solomon_from_coeff_vecs(&self, mut coeff_vecs: Vec<Vec<F>>) -> Vec<Vec<F>> {
        let mut res = vec![];
        for mut coeffs in coeff_vecs.iter_mut() {
            plonky2_util::reverse_index_bits_in_place(&mut coeffs);
            let extended_length = coeffs.len() << self.rate_bits;
            coeffs.resize(extended_length, F::zero());
            res.push(F::fft(&coeffs));
        }
        res

        // ======================================
        // todo: implement batch operation
        // ======================================
        // let length = coeff_vecs[0].len();
        // let num_poly = coeff_vecs.len();
        // let extended_length = length << self.rate_bits;

        // let timer = start_timer!(|| "reverse index bits in batch rs code");
        // coeff_vecs.iter_mut().for_each(|coeffs| {
        //     plonky2_util::reverse_index_bits_in_place(coeffs);
        // });
        // end_timer!(timer);

        // let timer = start_timer!(|| "dft in batch rs code");
        // // transpose the vector to make it suitable for batch dft
        // // somehow manually transpose the vector is faster than DenseMatrix.transpose()
        // let mut buf = vec![F::zero(); coeff_vecs.len() * extended_length];
        // coeff_vecs.iter().enumerate().for_each(|(i, coeffs)| {
        //     coeffs.iter().enumerate().for_each(|(j, &coeff)| {
        //         buf[num_poly * j + i] = coeff;
        //     });
        // });
        // drop(coeff_vecs);

        // let dft_res = p3_dft::Radix2DitParallel
        //     .dft_batch(RowMajorMatrix::new(buf, num_poly))
        //     .to_row_major_matrix()
        //     .values;
        // end_timer!(timer);

        // let timer = start_timer!(|| "transpose vector in batch rs code");
        // // somehow manually transpose the vector is faster than DenseMatrix.transpose()
        // let mut res = vec![Vec::with_capacity(extended_length); num_poly];
        // res.iter_mut().enumerate().for_each(|(i, r)| {
        //     dft_res.chunks_exact(num_poly).for_each(|chunk| {
        //         r.push(chunk[i]);
        //     });
        // });
        // end_timer!(timer);
        // res
    }

    fn batch_basefold_oracle_from_slices(&self, evals: &[&[F]]) -> Vec<Tree<F>> {
        let timer = start_timer!(|| "interpolate over hypercube");
        let coeffs: Vec<Vec<F>> = evals
            .iter()
            .map(|&evals| MultiLinearPoly::interpolate_over_hypercube_impl(evals))
            .collect();
        end_timer!(timer);

        let timer = start_timer!(|| "batch rs from coeffs");
        let rs_codes = self.batch_reed_solomon_from_coeff_vecs(coeffs);
        end_timer!(timer);

        let timer = start_timer!(|| "new from leaves");

        let trees = rs_codes
            .iter()
            .map(|codeword| Tree::new_with_field_elements(codeword))
            .collect::<Vec<_>>();
        end_timer!(timer);
        trees
    }

    pub fn basefold_oracle_from_poly(&self, poly: &MultiLinearPoly<F>) -> Tree<F> {
        let timer =
            start_timer!(|| format!("basefold oracle from poly of {} vars", poly.get_num_vars()));
        let timer2 = start_timer!(|| "interpolate over hypercube");
        let coeffs = poly.interpolate_over_hypercube();
        end_timer!(timer2);

        let timer2 = start_timer!(|| "reed solomon from coeffs");
        let codeword = self.reed_solomon_from_coeffs(coeffs);
        end_timer!(timer2);

        let timer2 = start_timer!(|| "new from leaves");
        let tree = Tree::new_with_field_elements(&codeword);
        end_timer!(timer2);
        end_timer!(timer);
        tree
    }
}
