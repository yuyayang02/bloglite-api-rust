#![allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};

enum TEST {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("one to one", |b| {
        b.iter_batched(
            || TEST::A,
            |t| match t {
                TEST::A => 0,
                TEST::B => 0,
                TEST::C => 0,
                TEST::D => 0,
                TEST::E => 1,
                TEST::F => 1,
                TEST::G => 1,
                TEST::H => 1,
            },
            criterion::BatchSize::SmallInput,
        );
    });

    c.bench_function("many to one", |b| {
        b.iter_batched(
            || TEST::A,
            |t| match t {
                TEST::A | TEST::B | TEST::C | TEST::D => 0,
                TEST::E | TEST::F | TEST::G | TEST::H => 1,
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
