use arith::Field;
use babybear::BabyBearx16;
use criterion::{criterion_group, criterion_main, Criterion};
use mersenne31::M31x16;
use poseidon::{PoseidonBabyBearParams, PoseidonBabyBearState};
use poseidon::{PoseidonM31Params, PoseidonM31State};

criterion_group!(benches, bench_poseidon_m31, bench_poseidon_babybear);
criterion_main!(benches);

const REPEAT: usize = 1000;

fn bench_poseidon_m31(c: &mut Criterion) {
    c.bench_function("poseidon_m31", |b| {
        let mut rng = rand::thread_rng();
        let param = PoseidonM31Params::new(&mut rand::thread_rng());
        let mut state = PoseidonM31State {
            state: M31x16::random_unsafe(&mut rng),
        };

        b.iter(|| {
            (0..REPEAT).for_each(|_| param.permute(&mut state));
        });
    });
}
fn bench_poseidon_babybear(c: &mut Criterion) {
    c.bench_function("poseidon_babybear", |b| {
        let mut rng = rand::thread_rng();
        let param = PoseidonBabyBearParams::new(&mut rand::thread_rng());
        let mut state = PoseidonBabyBearState {
            state: BabyBearx16::random_unsafe(&mut rng),
        };

        b.iter(|| {
            (0..REPEAT).for_each(|_| param.permute(&mut state));
        });
    });
}
