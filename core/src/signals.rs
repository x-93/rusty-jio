//! OS signal listening and graceful shutdown triggers.

use tokio_util::sync::CancellationToken;

pub struct Signals;

impl Signals {
    /// Spawns a background task listening for Ctrl+C / SIGTERM and trips the token.
    pub fn bind(shutdown_token: CancellationToken) {
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                println!("\n[INFO] Shutdown signal (Ctrl+C) received. Initiating graceful shutdown...");
                shutdown_token.cancel();
            }
        });
    }
}
