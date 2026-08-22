use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use jio_hashes::{
    BlockHash, Hash, Hasher, HasherBase, KHeavyHash, MerkleBranchHash, PersonalMessageSigningHash,
    PowHash, TransactionHash, TransactionID, TransactionSigningHash,
};
use std::str::FromStr;

fn bench_domain_hashers(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain_separated_hashing");

    let small_payload = [0x55u8; 32];
    let tx_payload = [0xaau8; 1024]; // 1 KB typical transaction
    let block_payload = [0x42u8; 32 * 1024]; // 32 KB block payload

    group.bench_function("transaction_hash_1kb", |b| {
        b.iter(|| black_box(TransactionHash::hash(black_box(&tx_payload))))
    });

    group.bench_function("transaction_id_1kb", |b| {
        b.iter(|| black_box(TransactionID::hash(black_box(&tx_payload))))
    });

    group.bench_function("transaction_signing_hash_1kb", |b| {
        b.iter(|| black_box(TransactionSigningHash::hash(black_box(&tx_payload))))
    });

    group.bench_function("block_hash_header_256b", |b| {
        let header_bytes = [0x77u8; 256];
        b.iter(|| black_box(BlockHash::hash(black_box(&header_bytes))))
    });

    group.bench_function("merkle_branch_hash", |b| {
        let left = Hash::from([1u8; 32]);
        let right = Hash::from([2u8; 32]);
        b.iter(|| {
            let mut hasher = MerkleBranchHash::new();
            hasher.update(left.as_bytes());
            hasher.update(right.as_bytes());
            black_box(hasher.finalize())
        })
    });

    group.bench_function("personal_message_signing_hash", |b| {
        b.iter(|| black_box(PersonalMessageSigningHash::hash(black_box(&small_payload))))
    });

    group.bench_function("block_payload_32kb", |b| {
        b.iter(|| black_box(TransactionHash::hash(black_box(&block_payload))))
    });

    group.finish();
}

fn bench_throughput_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_throughput_scaling");

    for size in [64, 256, 1024, 8192, 65536, 1048576].iter() {
        let data = vec![0x33u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(TransactionHash::hash(black_box(&data))))
        });
    }

    group.finish();
}

fn bench_hash_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_primitives");

    let raw_bytes = [
        0x8eu8, 0x40, 0xaf, 0x02, 0x26, 0x53, 0x60, 0xd5, 0x9f, 0x4e, 0xcf, 0x9a, 0xe9, 0xeb, 0xf8,
        0xf0, 0x0a, 0x31, 0x18, 0x40, 0x8f, 0x5a, 0x9c, 0xdc, 0xbc, 0xc9, 0xc0, 0xf9, 0x36, 0x42,
        0xf3, 0xaf,
    ];
    let hash = Hash::from(raw_bytes);
    let hex_str = "8e40af02265360d59f4ecf9ae9ebf8f00a3118408f5a9cdcbcc9c0f93642f3af";

    group.bench_function("to_le_u64_words", |b| {
        b.iter(|| black_box(hash.to_le_u64()))
    });

    group.bench_function("from_le_u64_words", |b| {
        let words = hash.to_le_u64();
        b.iter(|| black_box(Hash::from_le_u64(black_box(words))))
    });

    group.bench_function("faster_hex_display", |b| {
        b.iter(|| black_box(hash.to_string()))
    });

    group.bench_function("faster_hex_from_str_parsing", |b| {
        b.iter(|| black_box(Hash::from_str(black_box(hex_str)).unwrap()))
    });

    group.finish();
}

fn bench_proof_of_work(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_of_work_hashers");

    let pre_pow = [0xbbu8; 32];
    let nonce = 0x123456789abcdef0u64;

    group.bench_function("pow_hash_finalization", |b| {
        b.iter(|| {
            let mut hasher = PowHash::new();
            hasher.update(pre_pow);
            hasher.update(nonce.to_le_bytes());
            black_box(hasher.finalize())
        })
    });

    group.bench_function("kheavy_hash_finalization", |b| {
        b.iter(|| {
            let mut hasher = KHeavyHash::new();
            hasher.update(pre_pow);
            hasher.update(nonce.to_le_bytes());
            black_box(hasher.finalize())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_domain_hashers,
    bench_throughput_scaling,
    bench_hash_primitives,
    bench_proof_of_work
);
criterion_main!(benches);
