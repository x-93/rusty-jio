use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use jio_math::{Uint128, Uint192, Uint256};

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("uint256_arithmetic");

    let a = Uint256::from_limbs([
        0x1122334455667788,
        0x99aabbccddeeff00,
        0x0123456789abcdef,
        0xfedcba9876543210,
    ]);
    let b = Uint256::from_limbs([
        0x0f0e0d0c0b0a0908,
        0x0706050403020100,
        0x8877665544332211,
        0x00ffeeddccbbaa99,
    ]);

    group.bench_function("add_carrying", |bench| {
        bench.iter(|| {
            let (res, carry) = a.carrying_add(b);
            black_box((res, carry))
        })
    });

    group.bench_function("sub_borrowing", |bench| {
        bench.iter(|| {
            let (res, borrow) = a.borrowing_sub(b);
            black_box((res, borrow))
        })
    });

    group.bench_function("mul_carrying", |bench| {
        bench.iter(|| {
            let (res, carry) = a.carrying_mul(b);
            black_box((res, carry))
        })
    });

    group.bench_function("shift_left_1_bit", |bench| {
        bench.iter(|| black_box(a << 1usize))
    });

    group.bench_function("shift_right_1_bit", |bench| {
        bench.iter(|| black_box(a >> 1usize))
    });

    group.bench_function("shift_right_65_bits_across_limbs", |bench| {
        bench.iter(|| black_box(a >> 65usize))
    });

    group.bench_function("ord_comparison_pow_target", |bench| {
        bench.iter(|| black_box(a.cmp(&b)))
    });

    group.finish();
}

fn bench_multi_precision_tiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_precision_tiers");

    let u128_a = Uint128::from_limbs([0x1122334455667788, 0x99aabbccddeeff00]);
    let u128_b = Uint128::from_limbs([0x0f0e0d0c0b0a0908, 0x0706050403020100]);

    let u192_a = Uint192::from_limbs([0x1122334455667788, 0x99aabbccddeeff00, 0x0123456789abcdef]);
    let u192_b = Uint192::from_limbs([0x0f0e0d0c0b0a0908, 0x0706050403020100, 0x8877665544332211]);

    group.bench_function("uint128_mul", |bench| {
        bench.iter(|| black_box(u128_a.carrying_mul(u128_b)))
    });

    group.bench_function("uint192_mul", |bench| {
        bench.iter(|| black_box(u192_a.carrying_mul(u192_b)))
    });

    group.finish();
}

fn bench_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("uint256_conversions");

    let val = Uint256::from_limbs([
        0x1122334455667788,
        0x99aabbccddeeff00,
        0x0123456789abcdef,
        0xfedcba9876543210,
    ]);

    group.bench_function("to_le_bytes", |bench| {
        bench.iter(|| black_box(val.to_le_bytes()))
    });

    group.bench_function("from_le_bytes", |bench| {
        let bytes = val.to_le_bytes();
        bench.iter(|| black_box(Uint256::from_le_bytes(black_box(bytes))))
    });

    group.bench_function("to_be_bytes", |bench| {
        bench.iter(|| black_box(val.to_be_bytes()))
    });

    group.bench_function("from_be_bytes", |bench| {
        let bytes = val.to_be_bytes();
        bench.iter(|| black_box(Uint256::from_be_bytes(black_box(bytes))))
    });

    group.bench_function("to_f64_daa_window", |bench| {
        bench.iter(|| black_box(val.to_f64()))
    });

    group.bench_function("from_f64_saturating", |bench| {
        let f = 1.23456789e18;
        bench.iter_batched(|| f, Uint256::from_f64_saturating, BatchSize::SmallInput);
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_multi_precision_tiers,
    bench_conversions
);
criterion_main!(benches);
