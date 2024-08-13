// this module benchmarks the performance of different field operations

use arith::{Field, M31Ext3, SimdGF2_128, SimdM31, SimdM31Ext3, GF2, M31};
use ark_std::test_rng;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use halo2curves::bn256::Fr;
use tynm::type_name;

fn random_element<F: Field>() -> F {
    let mut rng = test_rng();
    F::random_unsafe(&mut rng)
}

pub(crate) fn bench_field<F: Field>(c: &mut Criterion) {
    c.bench_function(
        &format!(
            "mul-throughput<{}> 100x times {}x ",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || {
                    (
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                    )
                },
                |(mut x, mut y, mut z, mut w)| {
                    for _ in 0..25 {
                        (x, y, z, w) = (x * y, y * z, z * w, w * x);
                    }
                    (x, y, z, w)
                },
                BatchSize::SmallInput,
            )
        },
    );

    c.bench_function(
        &format!(
            "mul-latency<{}> 100x times {}x ",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || random_element::<F>(),
                |mut x| {
                    for _ in 0..100 {
                        x = x * x;
                    }
                    x
                },
                BatchSize::SmallInput,
            )
        },
    );

    c.bench_function(
        &format!(
            "sqr-throughput<{}> 100x times {}x",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || {
                    (
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                    )
                },
                |(mut x, mut y, mut z, mut w)| {
                    for _ in 0..25 {
                        (x, y, z, w) = (x.square(), y.square(), z.square(), w.square());
                    }
                    (x, y, z, w)
                },
                BatchSize::SmallInput,
            )
        },
    );

    c.bench_function(
        &format!(
            "sqr-latency<{}> 100x times {}x",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || random_element::<F>(),
                |mut x| {
                    for _ in 0..100 {
                        x = x.square();
                    }
                    x
                },
                BatchSize::SmallInput,
            )
        },
    );

    c.bench_function(
        &format!(
            "add-throughput<{}> 100x times {}x",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || {
                    (
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                        random_element::<F>(),
                    )
                },
                |(mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h, mut i, mut j)| {
                    for _ in 0..10 {
                        (a, b, c, d, e, f, g, h, i, j) = (
                            a + b,
                            b + c,
                            c + d,
                            d + e,
                            e + f,
                            f + g,
                            g + h,
                            h + i,
                            i + j,
                            j + a,
                        );
                    }
                    (a, b, c, d, e, f, g, h, i, j)
                },
                BatchSize::SmallInput,
            )
        },
    );

    c.bench_function(
        &format!(
            "add-latency<{}> 100x times {}x",
            type_name::<F>(),
            F::SIZE * 8 / F::FIELD_SIZE
        ),
        |b| {
            b.iter_batched(
                || random_element::<F>(),
                |mut x| {
                    for _ in 0..100 {
                        x = x + x;
                    }
                    x
                },
                BatchSize::SmallInput,
            )
        },
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_field::<M31>(c);
    bench_field::<SimdM31>(c);
    bench_field::<M31Ext3>(c);
    bench_field::<SimdM31Ext3>(c);
    bench_field::<Fr>(c);
    bench_field::<GF2>(c);
    bench_field::<SimdGF2_128>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
