use clap::Parser;
use jio_daemon::args::DaemonArgs;
use jio_daemon::daemon::JioDaemon;
use std::io::Write;

fn init_logger() {
    let mut builder = env_logger::Builder::from_default_env();
    if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(log::LevelFilter::Info);
    }
    builder
        .format(|buf, record| {
            let ts = buf.timestamp_millis();
            let level = format!("{:<5}", record.level());
            let module = match record.module_path() {
                Some(p) if p.contains("consensus") => "CONSENSUS",
                Some(p) if p.contains("mining") => "MINING",
                Some(p) if p.contains("p2p") || p.contains("protocol") => "P2P",
                Some(p) if p.contains("rpc") || p.contains("grpc") || p.contains("wrpc") => "RPC",
                Some(p) if p.contains("utxo") || p.contains("indexes") => "INDEXER",
                Some(p) if p.contains("perf") => "PROFILER",
                _ => "JIOPAD",
            };
            writeln!(buf, "{} [{}] [{}]: {}", ts, level, module, record.args())
        })
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    let args = DaemonArgs::parse();

    log::info!("Application version {}", env!("CARGO_PKG_VERSION"));
    log::info!("Target network: {}", args.network);

    let daemon = JioDaemon::new(args);
    daemon.run().await?;
    Ok(())
}
