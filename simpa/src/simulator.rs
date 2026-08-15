use crate::generator::DagBlockGenerator;
use jio_consensus::consensus::factory::ConsensusFactory;
use jio_consensus_core::config::params::Params;
use jio_consensusmanager::ConsensusManager;
use std::time::Instant;

pub struct DagSimulator;

impl DagSimulator {
    pub fn simulate_bps(bps: u64, duration_secs: u64) -> (u64, f64) {
        println!("Initializing consensus engine for DAG simulation...");
        let params = Params::devnet();
        let consensus = ConsensusFactory::new_instance(params);
        let consensus_manager = ConsensusManager::new(consensus);

        let total_blocks = bps * duration_secs;
        println!("Simulating {} blocks at target {} BPS (duration: {}s)...", total_blocks, bps, duration_secs);

        let start = Instant::now();
        let mut inserted = 0u64;

        for i in 0..total_blocks {
            let session = consensus_manager.session();
            let vs = session.get_virtual_state().expect("virtual state");
            let parents = vs.parents.clone();
            let block = DagBlockGenerator::create_synthetic_block(
                parents,
                i + 1,
                1680000000000 + (i + 1) * 1000,
                0x207f_ffff,
                i,
            );

            if let Ok(_hash) = session.validate_and_insert_block(block) {
                inserted += 1;
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let actual_bps = if elapsed > 0.0 { inserted as f64 / elapsed } else { 0.0 };

        println!("Simulation Complete!");
        println!("- Blocks Processed: {} / {}", inserted, total_blocks);
        println!("- Time Elapsed:     {:.3} seconds", elapsed);
        println!("- Effective BPS:    {:.2} blocks/sec", actual_bps);

        (inserted, actual_bps)
    }
}
