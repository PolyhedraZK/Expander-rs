use std::mem::transmute;

use arith::Field;
use ark_std::{end_timer, start_timer};
use mersenne31::{M31x16, M31};
use rand::RngCore;

#[derive(Clone, Debug, Default)]
pub struct PoseidonM31State {
    pub state: M31x16,
}

#[derive(Clone, Debug, Default)]
pub struct PoseidonM31Params {
    pub full_rounds: usize,
    pub partial_rounds: usize,
    pub sbox: usize,
    // the mds matrix is a 16x16 matrix of M31 elements
    pub mds: [M31x16; 16],
    // for each round the key is 16 M31 elements
    pub round_constants: Vec<M31x16>,
}

impl PoseidonM31Params {
    pub fn new(mut rng: impl RngCore) -> Self {
        let full_rounds = 8;
        let partial_rounds = 14;
        let sbox = 5;
        let mds = (0..16)
            .map(|_| M31x16::random_unsafe(&mut rng))
            .collect::<Vec<M31x16>>();
        let round_constants = (0..full_rounds + partial_rounds)
            .map(|_| M31x16::random_unsafe(&mut rng))
            .collect::<Vec<M31x16>>();
        Self {
            full_rounds,
            partial_rounds,
            sbox,
            mds: mds.try_into().unwrap(),
            round_constants,
        }
    }

    #[inline]
    pub fn permute(&self, state: &mut PoseidonM31State) {
        let mut state = state.state;
        let half_full_rounds = self.full_rounds >> 1;

        // Applies the full rounds.
        for i in 0..half_full_rounds {
            // add round constants
            Self::add_round_constant(&mut state, &self.round_constants[i]);
            // apply the mds matrix
            self.apply_mds(&mut state);
            // apply the sbox
            Self::full_round_sbox(&mut state);
        }

        // Applies the partial rounds.
        for i in half_full_rounds..half_full_rounds + self.partial_rounds {
            // add round constants
            Self::add_round_constant(&mut state, &self.round_constants[i]);
            // apply the mds matrix
            self.apply_mds(&mut state);
            // apply the sbox
            Self::partial_round_sbox(&mut state);
        }

        // Applies the full rounds.
        for i in half_full_rounds + self.partial_rounds..self.full_rounds + self.partial_rounds {
            // add round constants
            Self::add_round_constant(&mut state, &self.round_constants[i]);
            // apply the mds matrix
            self.apply_mds(&mut state);
            // apply the sbox
            Self::full_round_sbox(&mut state);
        }
    }

    #[inline]
    fn add_round_constant(state: &mut M31x16, round_key: &M31x16) {
        let timer = start_timer!(|| "add round constants");
        *state += *round_key;
        end_timer!(timer);
    }

    #[inline]
    fn apply_mds(&self, state: &mut M31x16) {
        let timer = start_timer!(|| "apply_mds");
        let mut res = [0u32; 16];

        res.iter_mut().zip(self.mds.iter()).for_each(|(res, mds)| {
            let timer1 = start_timer!(|| "pair product");
            let pair_product = *mds * *state;
            end_timer!(timer1);

            let timer1 = start_timer!(|| "sum");
            let sum = unsafe {
                transmute::<M31x16, [u32; 16]>(pair_product)
                    .iter()
                    .map(|&x| x as u64)
                    .sum::<u64>()
            };
            *res = mod_reduce_u64_safe(sum);
            end_timer!(timer1);
        });

        // parallelize the code make it 10x slower SMH...
        // It is likely because within each thread we only did 1 AVXM31 mul and 1 sum
        // The overhead of creating threads and managing them is too high
        //
        // use rayon::iter::{
        //     IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator,
        //     ParallelIterator,
        // };
        //
        // res.par_iter_mut()
        //     .zip_eq(self.mds.par_iter())
        //     .for_each(|(res, mds)| {
        //         let pair_product = *mds * *state;
        //         let sum = unsafe {
        //             transmute::<_, [u32; 16]>(pair_product)
        //                 .iter()
        //                 .map(|&x| x as u64)
        //                 .sum::<u64>()
        //         };
        //         *res = mod_reduce_u64(sum);
        //     });

        *state = unsafe { transmute::<[u32; 16], M31x16>(res) };
        end_timer!(timer);
    }

    #[inline]
    fn full_round_sbox(state: &mut M31x16) {
        let timer = start_timer!(|| "full_round_sbox");
        let double = *state * *state;
        let quad = double * double;
        *state *= quad;
        end_timer!(timer);
    }

    #[inline]
    fn partial_round_sbox(state: &mut M31x16) {
        let time = start_timer!(|| "partial_round_sbox");
        let mut buf = unsafe { transmute::<M31x16, [u32; 16]>(*state) };
        let e = M31 { v: buf[0] };
        let e2 = e * e;
        let e4 = e2 * e2;
        let e5 = e4 * e;
        buf[0] = e5.v;
        *state = unsafe { transmute::<[u32; 16], M31x16>(buf) };
        end_timer!(time);
    }
}

const M31_MOD: u32 = 2147483647;

#[inline]
// mod reduce u64 with a promise the input is less than (2^31-1) * 16
fn mod_reduce_u64_safe(x: u64) -> u32 {
    let mut t = (x & M31_MOD as u64) + (x >> 31);
    if t >= M31_MOD as u64 {
        t -= M31_MOD as u64;
    }
    t as u32
}
