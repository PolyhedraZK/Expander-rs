use ark_std::log2;

use crate::Field;

pub trait FFTField: Field {
    const TWO_ADICITY: u32;

    /// Returns a primitive $n$-th root of unity.
    fn root_of_unity() -> Self;

    /// Returns a generator of the multiplicative group of order `2^bits`.
    /// Assumes `bits < TWO_ADICITY`, otherwise the result is undefined.
    #[must_use]
    fn two_adic_generator(bits: usize) -> Self;

    #[inline]
    fn fft(poly: &[Self]) -> Vec<Self> {
        let log_degree = log2(poly.len()) as usize;
        let omega = &Self::two_adic_generator(log_degree);

        let mut poly = poly.to_vec();
        Self::fft_in_place(&mut poly, omega);
        poly
    }

    #[inline]
    fn ifft(poly: &[Self]) -> Vec<Self> {
        let log_degree = log2(poly.len()) as usize;
        let omega = &Self::two_adic_generator(log_degree);
        let omega_inv = omega.inv().unwrap();

        let mut poly = poly.to_vec();
        Self::ifft_in_place(&mut poly, &omega_inv);
        poly
    }

    #[inline]
    fn fft_in_place(poly: &mut [Self], omega: &Self) {
        let log_n = log2(poly.len());
        single_thread_fft(poly, omega, log_n);
    }

    #[inline]
    fn ifft_in_place(poly: &mut [Self], omega_inv: &Self) {
        let log_n = log2(poly.len());
        let n = poly.len();
        let n_inv = Self::from(n as u32).inv().unwrap();
        single_thread_fft(poly, omega_inv, log_n);
        poly.iter_mut().for_each(|x| *x *= n_inv);
    }
}

/// Performs a radix-$2$ Fast-Fourier Transformation (FFT) on a vector of size
/// $n = 2^k$, when provided `log_n` = $k$ and an element of multiplicative
/// order $n$ called `omega` ($\omega$). The result is that the vector `a`, when
/// interpreted as the coefficients of a polynomial of degree $n - 1$, is
/// transformed into the evaluations of this polynomial at each of the $n$
/// distinct powers of $\omega$. This transformation is invertible by providing
/// $\omega^{-1}$ in place of $\omega$ and dividing each resulting field element
/// by $n$.
///
/// This will use multithreading if beneficial.
pub fn single_thread_fft<F: FFTField>(a: &mut [F], omega: &F, log_n: u32) {
    let n = a.len();
    assert_eq!(n, 1 << log_n);

    for k in 0..n {
        let rk = bitreverse(k, log_n as usize);
        if k < rk {
            a.swap(rk, k);
        }
    }

    let mut m = 1;
    for _ in 0..log_n {
        let w_m = omega.exp((n / (2 * m)) as u128);

        let mut k = 0;
        while k < n {
            let mut w = F::ONE;
            for j in 0..m {
                let mut t = a[(k + j + m) as usize];
                t *= &w;
                a[(k + j + m) as usize] = a[(k + j) as usize];
                a[(k + j + m) as usize] -= &t;
                a[(k + j) as usize] += &t;
                w *= &w_m;
            }

            k += 2 * m;
        }

        m *= 2;
    }
}

#[inline]
fn bitreverse(mut n: usize, l: usize) -> usize {
    let mut r = 0;
    for _ in 0..l {
        r = (r << 1) | (n & 1);
        n >>= 1;
    }
    r
}
