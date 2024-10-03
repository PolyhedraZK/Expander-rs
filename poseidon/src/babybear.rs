use std::mem::transmute;

use arith::Field;
use ark_std::{end_timer, start_timer};
use babybear::{BabyBear, BabyBearx16};
use rand::RngCore;

#[derive(Clone, Debug, Default)]
pub struct PoseidonBabyBearState {
    pub state: BabyBearx16,
}

#[derive(Clone, Debug, Default)]
pub struct PoseidonBabyBearParams {
    pub full_rounds: usize,
    pub partial_rounds: usize,
    pub sbox: usize,
    // the mds matrix is a 16x16 matrix of M31 elements
    pub mds: [BabyBearx16; 16],
    // for each round the key is 16 M31 elements
    pub round_constants: Vec<BabyBearx16>,
}

impl PoseidonBabyBearParams {
    pub fn new(mut rng: impl RngCore) -> Self {
        let full_rounds = 8;
        let partial_rounds = 14;
        let sbox = 5;
        let mds = (0..16)
            .map(|_| BabyBearx16::random_unsafe(&mut rng))
            .collect::<Vec<BabyBearx16>>();
        let round_constants = (0..full_rounds + partial_rounds)
            .map(|_| BabyBearx16::random_unsafe(&mut rng))
            .collect::<Vec<BabyBearx16>>();
        Self {
            full_rounds,
            partial_rounds,
            sbox,
            mds: mds.try_into().unwrap(),
            round_constants,
        }
    }

    #[inline]
    pub fn permute(&self, state: &mut PoseidonBabyBearState) {
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
    fn add_round_constant(state: &mut BabyBearx16, round_key: &BabyBearx16) {
        let timer = start_timer!(|| "add round constants");
        *state += *round_key;
        end_timer!(timer);
    }

    #[inline]
    fn apply_mds(&self, state: &mut BabyBearx16) {
        let timer = start_timer!(|| "apply_mds");
        let mut res = [0u32; 16];

        res.iter_mut().zip(self.mds.iter()).for_each(|(res, mds)| {
            let timer1 = start_timer!(|| "pair product");
            let pair_product = *mds * *state;
            end_timer!(timer1);

            let timer1 = start_timer!(|| "sum");
            *res = unsafe {
                transmute::<BabyBear, u32>(
                    transmute::<BabyBearx16, [BabyBear; 16]>(pair_product)
                        .iter()
                        .sum::<BabyBear>(),
                )
            };
            end_timer!(timer1);
        });

        *state = unsafe { transmute::<[u32; 16], BabyBearx16>(res) };
        end_timer!(timer);
    }

    #[inline]
    fn full_round_sbox(state: &mut BabyBearx16) {
        let timer = start_timer!(|| "full_round_sbox");
        let double = *state * *state;
        let quad = double * double;
        *state *= double * quad;
        end_timer!(timer);
    }

    #[inline]
    fn partial_round_sbox(state: &mut BabyBearx16) {
        let time = start_timer!(|| "partial_round_sbox");
        let mut buf = unsafe { transmute::<BabyBearx16, [u32; 16]>(*state) };
        let e = unsafe { transmute::<u32, BabyBear>(buf[0]) };
        let e2 = e * e;
        let e4 = e2 * e2;
        let e5 = e4 * e;
        buf[0] = unsafe { transmute::<BabyBear, u32>(e5) };
        *state = unsafe { transmute::<[u32; 16], BabyBearx16>(buf) };
        end_timer!(time);
    }
}
