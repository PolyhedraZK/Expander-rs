#![cfg_attr(target_arch = "x86_64", feature(stdarch_x86_avx512))]

pub mod poly_commit;
pub use poly_commit::*;

pub mod prover;
pub use prover::*;

pub mod verifier;
pub use verifier::*;

pub mod utils;

#[cfg(test)]
mod tests;
