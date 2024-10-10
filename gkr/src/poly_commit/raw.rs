//! RAW commitment refers to the case where the prover does not commit to the witness at all.
//! The prover will send the whole witnesses to the verifier.

use std::io::{Read, Write};

use arith::{Field, FieldSerde, FieldSerdeResult, SimdField};
use config::{GKRConfig, MPIConfig};
use polynomials::MultiLinearPoly;

#[derive(Default)]
pub struct RawOpening {}

pub struct RawCommitment<C: GKRConfig> {
    pub poly_vals: Vec<C::SimdCircuitField>,
}

impl<C: GKRConfig> RawCommitment<C> {
    #[inline]
    pub fn size(&self) -> usize {
        self.poly_vals.len() * C::SimdCircuitField::SIZE
    }

    #[inline]
    pub fn serialize_into<W: Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        self.poly_vals
            .iter()
            .try_for_each(|v| v.serialize_into(&mut writer))
    }

    #[inline]
    pub fn deserialize_from<R: Read>(mut reader: R, poly_size: usize) -> Self {
        let poly_vals = (0..poly_size)
            .map(|_| C::SimdCircuitField::deserialize_from(&mut reader).unwrap()) // TODO: error propagation
            .collect();

        RawCommitment { poly_vals }
    }
}

impl<C: GKRConfig> RawCommitment<C> {
    #[inline]
    pub fn new(poly_vals: &[C::SimdCircuitField]) -> Self {
        RawCommitment {
            poly_vals: poly_vals.to_owned(),
        }
    }

    /// create a commitment collectively
    /// Should also work if mpi is not initialized
    #[inline]
    pub fn mpi_new(local_poly_vals: &Vec<C::SimdCircuitField>, mpi_config: &MPIConfig) -> Self {
        if mpi_config.world_size() == 1 {
            Self::new(local_poly_vals)
        } else {
            let mut buffer = if mpi_config.is_root() {
                vec![C::SimdCircuitField::zero(); local_poly_vals.len() * mpi_config.world_size()]
            } else {
                vec![]
            };

            mpi_config.gather_vec(local_poly_vals, &mut buffer);
            Self { poly_vals: buffer }
        }
    }

    #[inline(always)]
    fn eval_local(
        v: &[C::SimdCircuitField],
        x: &[C::ChallengeField],
        x_simd: &[C::ChallengeField],
    ) -> C::ChallengeField {
        let mut scratch = vec![C::Field::default(); v.len()];
        let y_simd = C::eval_circuit_vals_at_challenge(v, x, &mut scratch);
        let y_simd_unpacked = y_simd.unpack();
        let mut scratch = vec![C::ChallengeField::default(); y_simd_unpacked.len()];
        MultiLinearPoly::evaluate_with_buffer(&y_simd_unpacked, x_simd, &mut scratch)
    }

    #[inline]
    pub fn verify(
        &self,
        x: &[C::ChallengeField],
        x_simd: &[C::ChallengeField],
        y: C::ChallengeField,
    ) -> bool {
        y == Self::eval_local(&self.poly_vals, x, x_simd)
    }

    /// Note: this only runs on the root rank
    /// Note: verifier should not have access to the MPIConfig
    /// The mpi size is implicitly specified by the length of x_mpi,
    /// So make sure it is correct
    #[inline]
    pub fn mpi_verify(
        &self,
        x: &[C::ChallengeField],
        x_simd: &[C::ChallengeField],
        x_mpi: &[C::ChallengeField],
        y: C::ChallengeField,
    ) -> bool {
        let local_poly_size = self.poly_vals.len() >> x_mpi.len();
        let local_evals = self
            .poly_vals
            .chunks(local_poly_size)
            .map(|local_vals| Self::eval_local(local_vals, x, x_simd))
            .collect::<Vec<C::ChallengeField>>();

        let mut scratch = vec![C::ChallengeField::default(); local_evals.len()];
        y == MultiLinearPoly::evaluate_with_buffer(&local_evals, x_mpi, &mut scratch)
    }
}
