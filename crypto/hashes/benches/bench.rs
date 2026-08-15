use jio_hashes::{blake3, HasherBase, TransactionHash};

fn main() {
    let data = b"Performance benchmark data sample for jio blake3 hashing";
    let start = std::time::Instant::now();
    let iterations = 100_000;

    for _ in 0..iterations {
        let _ = blake3(data);
    }
    let elapsed = start.elapsed();
    println!(
        "blake3 100k iterations: {:?} ({:.2} ops/sec)",
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );

    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut hasher = TransactionHash::new();
        hasher.update(data);
        let _ = hasher.finalize();
    }
    let elapsed = start.elapsed();
    println!(
        "TransactionHash 100k iterations: {:?} ({:.2} ops/sec)",
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );
}
