use jio_consensus_core::{
    header::Header,
    merkle::calc_hash_merkle_root,
    subnets::SUBNETWORK_ID_COINBASE,
    tx::Transaction,
};
use jio_hashes::{Hash, ZERO_HASH};
use jio_muhash::EMPTY_MUHASH;
use jio_pow::State;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let network_name = if args.len() > 1 { &args[1] } else { "jio-mainnet" };
    let custom_msg = if args.len() > 2 { &args[2] } else { "Jio Network Genesis Block 2026" };

    println!("=======================================================");
    println!("       Jio Custom Genesis Block Generator             ");
    println!("=======================================================");
    println!("Network Name:  {}", network_name);
    println!("Custom Message: {}", custom_msg);

    // 1. Timestamp in milliseconds
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    // 2. Compact target bits (difficulty)
    // 0x1e7fffff is easy enough to mine in seconds on CPU
    let bits: u32 = 0x1e7fffff;

    // 3. Construct custom coinbase payload
    let mut coinbase_payload = Vec::new();
    coinbase_payload.extend_from_slice(&0u64.to_le_bytes());        // Blue score (0)
    coinbase_payload.extend_from_slice(&100_000_000u64.to_le_bytes()); // Subsidy field: 1 JIO (100,000,000 sompis) - pure fair launch, outputs are empty
    coinbase_payload.extend_from_slice(&0u16.to_le_bytes());        // Script version (0)
    coinbase_payload.push(0x01);                                   // Varint
    coinbase_payload.push(0x00);                                   // OP_FALSE
    coinbase_payload.extend_from_slice(custom_msg.as_bytes());     // Custom payload string

    // 4. Build genesis transaction & calculate Merkle root
    let genesis_tx = Transaction::new(0, Vec::new(), Vec::new(), 0, SUBNETWORK_ID_COINBASE, 0, coinbase_payload.clone());
    let hash_merkle_root = calc_hash_merkle_root([&genesis_tx].into_iter(), false);

    // 5. Create template header
    let mut header = Header::new_finalized(
        0,                     // version
        Vec::new(),            // parents (empty for genesis)
        hash_merkle_root,      // hash_merkle_root
        ZERO_HASH,             // accepted_id_merkle_root
        EMPTY_MUHASH,          // utxo_commitment
        timestamp,             // timestamp
        bits,                  // bits
        0,                     // initial nonce
        0,                     // daa_score
        0.into(),              // blue_work
        0,                     // blue_score
        ZERO_HASH,             // pruning_point
    );

    // 6. Mine valid nonce using Proof of Work
    println!("Mining genesis block (bits: {:#010x})...", bits);
    let state = State::new(&header);
    let mut found_nonce = None;

    let start_time = std::time::Instant::now();
    for nonce in 0..u64::MAX {
        let (passed, _) = state.check_pow(nonce);
        if passed {
            found_nonce = Some(nonce);
            header.nonce = nonce;
            header.finalize();
            break;
        }
        if nonce % 1_000_000 == 0 && nonce > 0 {
            println!("  Searched {} nonces ({:.2?})...", nonce, start_time.elapsed());
        }
    }

    let nonce = found_nonce.expect("Failed to find valid nonce");
    println!("Found valid nonce: {:#x} in {:.2?}", nonce, start_time.elapsed());
    println!("Genesis Block Hash: {}", header.hash);
    println!("Hash Merkle Root:   {}", hash_merkle_root);
    println!("Timestamp:          {} ({})", timestamp, chrono::DateTime::from_timestamp_millis(timestamp as i64).unwrap_or_default());

    println!("\n=======================================================");
    println!(" Paste the following into consensus/core/src/config/genesis.rs");
    println!("=======================================================\n");

    println!("pub const GENESIS: GenesisBlock = GenesisBlock {{");
    print_hash_bytes("hash", &header.hash);
    println!("    version: 0,");
    print_hash_bytes("hash_merkle_root", &hash_merkle_root);
    println!("    utxo_commitment: EMPTY_MUHASH,");
    println!("    timestamp: {},", timestamp);
    println!("    bits: {:#010x},", bits);
    println!("    nonce: {:#x},", nonce);
    println!("    daa_score: 0,");
    println!("    #[rustfmt::skip]");
    println!("    coinbase_payload: &[");
    for chunk in coinbase_payload.chunks(8) {
        print!("        ");
        for b in chunk {
            print!("{:#04x}, ", b);
        }
        println!();
    }
    println!("    ],");
    println!("}};");
}

fn print_hash_bytes(field_name: &str, hash: &Hash) {
    println!("    {}: Hash::from_bytes([", field_name);
    let bytes = hash.as_bytes();
    for chunk in bytes.chunks(11) {
        print!("        ");
        for b in chunk {
            print!("{:#04x}, ", b);
        }
        println!();
    }
    println!("    ]),");
}
