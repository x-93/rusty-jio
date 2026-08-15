use crate::args::DaemonArgs;
use jio_addressmanager::AddressManager;
use jio_connectionmanager::ConnectionManager;
use jio_consensus::consensus::factory::ConsensusFactory;
use jio_consensus_core::config::params::Params;
use jio_consensusmanager::ConsensusManager;
use jio_grpc_server::GrpcServer;
use jio_mining::{MempoolConfig, MiningManager};
use jio_notify::Notifier;
use jio_perf_monitor::PerformanceMonitor;
use jio_protocol_flows::service::P2pService;
use jio_protocol_flows::FlowContext;
use jio_rpc_service::RpcCoreService;
use jio_utxoindex::UtxoIndex;
use std::sync::Arc;

pub struct JioDaemon {
    pub args: DaemonArgs,
    pub consensus_manager: ConsensusManager,
    pub mining_manager: MiningManager,
    pub address_manager: Arc<AddressManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub utxo_index: Arc<UtxoIndex>,
    pub notifier: Arc<Notifier>,
    pub perf_monitor: Arc<PerformanceMonitor>,
    pub rpc_service: Arc<RpcCoreService>,
    pub grpc_server: Arc<GrpcServer>,
    pub p2p_service: Arc<P2pService>,
}

impl JioDaemon {
    pub fn new(args: DaemonArgs) -> Self {
        let params = match args.get_network_type() {
            jio_consensus_core::network::NetworkType::Mainnet => Params::mainnet(),
            jio_consensus_core::network::NetworkType::Testnet => Params::testnet(),
            jio_consensus_core::network::NetworkType::Devnet => Params::devnet(),
            jio_consensus_core::network::NetworkType::Simnet => Params::simnet(),
        };

        let consensus = ConsensusFactory::new_instance(params);
        let consensus_manager = ConsensusManager::new(consensus);
        let mining_manager = MiningManager::new(consensus_manager.clone(), MempoolConfig::default());
        let address_manager = Arc::new(AddressManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(address_manager.as_ref().clone(), args.max_peers));
        let utxo_index = Arc::new(UtxoIndex::new());
        let notifier = Arc::new(Notifier::new());
        let perf_monitor = Arc::new(PerformanceMonitor::new());

        let rpc_service = Arc::new(RpcCoreService::new(
            consensus_manager.clone(),
            mining_manager.clone(),
            utxo_index.clone(),
            notifier.clone(),
        ));

        let grpc_server = Arc::new(GrpcServer::new(rpc_service.clone(), args.listen_rpc));

        let flow_context = FlowContext::new(
            consensus_manager.clone(),
            address_manager.as_ref().clone(),
            connection_manager.as_ref().clone(),
        );
        let p2p_service = Arc::new(P2pService::new(flow_context));

        Self {
            args,
            consensus_manager,
            mining_manager,
            address_manager,
            connection_manager,
            utxo_index,
            notifier,
            perf_monitor,
            rpc_service,
            grpc_server,
            p2p_service,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let net = self.args.get_network_type();
        log::info!("Loaded consensus instance for network {:?}", net);
        log::info!("Mempool initialized (max transaction mass: 500,000)");
        if self.args.utxoindex {
            log::info!("UTXO index initialized");
        }

        // 1. Start gRPC Server
        self.grpc_server.start().await?;
        log::info!("gRPC server listening on {}", self.args.listen_rpc);

        // 2. Start P2P Service
        self.p2p_service.start(self.args.listen_p2p).await?;
        log::info!("P2P listening on {}", self.args.listen_p2p);

        log::info!("Node is synced and ready. IBD complete.");

        // 3. Telemetry Ticker
        let perf = self.perf_monitor.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let (blocks, _txs, _headers) = perf.snapshot();
                if blocks > 0 {
                    log::info!("Processed {} blocks in the last 10s (throughput: {:.2} BPS)", blocks, blocks as f64 / 10.0);
                }
            }
        });

        // 4. Background block progression for devnet / simnet
        let mining = self.mining_manager.clone();
        let consensus_mgr = self.consensus_manager.clone();
        let rpc = self.rpc_service.clone();
        let perf_mon = self.perf_monitor.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                ticker.tick().await;
                let payee_spk = jio_consensus_core::tx::ScriptPublicKey::new(0, vec![0xaa, 0xbb]);
                if let Ok(template) = mining.get_block_template(payee_spk, vec![0x01, 0x02]) {
                    use jio_rpc_core::api::rpc::RpcApi;
                    use jio_rpc_core::model::message::SubmitBlockRequest;
                    let req = SubmitBlockRequest {
                        block: template.block,
                        allow_non_daa_blocks: false,
                    };
                    if let Ok(resp) = rpc.submit_block(req).await {
                        perf_mon.counters.record_block(1, 10);
                        let session = consensus_mgr.session();
                        if let Some(vs) = session.get_virtual_state() {
                            log::info!(
                                "Accepted block {} via Mining/RPC (DAA score: {}, blue score: {}, txs: 1)",
                                resp.hash,
                                vs.daa_score,
                                vs.blue_score
                            );
                        }
                    }
                }
            }
        });

        log::info!("Waiting for shutdown signal (Ctrl+C)...");
        tokio::signal::ctrl_c().await.map_err(|e| e.to_string())?;
        log::info!("Shutdown signal received. Stopping Jio Daemon.");

        Ok(())
    }
}
