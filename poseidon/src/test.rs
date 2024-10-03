use arith::Field;
use babybear::BabyBearx16;
use mersenne31::M31x16;

use crate::{PoseidonBabyBearParams, PoseidonBabyBearState, PoseidonM31Params, PoseidonM31State};

#[test]
fn test_poseidon_m31() {
    let mut rng = rand::thread_rng();
    let param = PoseidonM31Params::new(&mut rand::thread_rng());
    let mut state = PoseidonM31State {
        state: M31x16::random_unsafe(&mut rng),
    };

    param.permute(&mut state)
}

#[test]
fn test_poseidon_babybear() {
    let mut rng = rand::thread_rng();
    let param = PoseidonBabyBearParams::new(&mut rand::thread_rng());
    let mut state = PoseidonBabyBearState {
        state: BabyBearx16::random_unsafe(&mut rng),
    };

    param.permute(&mut state)
}
