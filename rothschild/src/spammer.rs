use crate::wallet::RothschildWallet;
use jio_addresses::Prefix;
use jio_hashes::Hash;
use std::time::Instant;

pub struct LoadSpammer;

impl LoadSpammer {
    pub fn spam_tps(target_tps: u64) -> (u64, f64) {
        println!("Initializing Rothschild Load Spammer...");
        let wallet = RothschildWallet::new(Prefix::Devnet);
        println!("Spammer test address: {}", wallet.address);

        let duration_secs = 3;
        let total_txs = target_tps * duration_secs;
        println!("Generating and dispatching {} transactions at target {} TPS...", total_txs, target_tps);

        let start = Instant::now();
        let mut prev_id = Hash::from([0xaa; 32]);
        let mut count = 0u64;

        for i in 0..total_txs {
            let tx = wallet.create_test_tx(prev_id, (i % 10) as u32, 100_000_000);
            prev_id = tx.id();
            count += 1;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let effective_tps = if elapsed > 0.0 { count as f64 / elapsed } else { 0.0 };

        println!("Stress Test Run Complete!");
        println!("- Transactions Dispatched: {}", count);
        println!("- Elapsed Time:            {:.3} seconds", elapsed);
        println!("- Effective Generation TPS:{:.2} tx/sec", effective_tps);

        (count, effective_tps)
    }
}
