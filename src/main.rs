//Log utilities for tracing and tracing-sub
pub mod log;
pub mod message;
pub mod network;
//Common use statements within this project modules
pub mod prelude {
    pub use std::error::Error;
    pub use tracing::{debug, error, info, instrument, trace, warn};
}

use crate::prelude::*;
use network::swarm::QPeer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::tracing_subscriber_setup("info").await;
    info!("quipu is starting...");

    let mut qpeer = QPeer::init().await?;

    qpeer.swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    qpeer.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    qpeer.run_swarm().await?;

    Ok(())
}

/// Parses in a very weird way the arguments
pub fn parse_cli() -> String {
    let mut args = std::env::args();
    _ = args.next();
    match args.find(|arg| arg.eq_ignore_ascii_case("dial") || arg.eq_ignore_ascii_case("listen")) {
        Some(val) => val,
        None => {
            error!("Wrong arguments, usage: <listen|dial> [target_multiaddress]");
            std::process::exit(1);
        }
    }
}
