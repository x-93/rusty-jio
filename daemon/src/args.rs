use clap::Parser;
use jio_consensus_core::network::NetworkType;
use std::net::SocketAddr;

#[derive(Parser, Clone, Debug)]
#[command(name = "jiod", about = "Jio Full Node Daemon", version)]
pub struct DaemonArgs {
    #[arg(long, default_value = "devnet", help = "Network type: mainnet, testnet, devnet, simnet")]
    pub network: String,

    #[arg(long, default_value = "127.0.0.1:16111", help = "P2P listening socket address")]
    pub listen_p2p: SocketAddr,

    #[arg(long, default_value = "127.0.0.1:16110", help = "RPC listening socket address")]
    pub listen_rpc: SocketAddr,

    #[arg(long, default_value_t = true, help = "Enable address UTXO index")]
    pub utxoindex: bool,

    #[arg(long, default_value_t = 128, help = "Maximum connected peers")]
    pub max_peers: usize,
}

impl DaemonArgs {
    pub fn get_network_type(&self) -> NetworkType {
        match self.network.to_lowercase().as_str() {
            "mainnet" => NetworkType::Mainnet,
            "testnet" => NetworkType::Testnet,
            "simnet" => NetworkType::Simnet,
            _ => NetworkType::Devnet,
        }
    }
}

impl Default for DaemonArgs {
    fn default() -> Self {
        Self {
            network: "devnet".to_string(),
            listen_p2p: "127.0.0.1:16111".parse().unwrap(),
            listen_rpc: "127.0.0.1:16110".parse().unwrap(),
            utxoindex: true,
            max_peers: 128,
        }
    }
}
