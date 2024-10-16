use quipu::{log, network::swarm::QPeer, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::tracing_subscriber_setup("info").await;
    info!("quipu is starting...");

    let mut qpeer = QPeer::init().await?;
    qpeer.run_swarm().await?;

    Ok(())
}
