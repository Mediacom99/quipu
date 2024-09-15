use futures::StreamExt;
use libp2p as p2;
use p2::Multiaddr;
use std::error::Error;
use tracing_subscriber::EnvFilter;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //This is for logging stuff I think
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // The swarm connects the Transport with the Network Behaviour
    let mut swarm = p2::SwarmBuilder::with_new_identity()
        .with_async_std()
        .with_tcp(
            p2::tcp::Config::default(), // tcp config
            p2::tls::Config::new,       // security upgrade
            p2::yamux::Config::default, // multiplexer upgrade
        )?
        .with_behaviour(|_| p2::ping::Behaviour::default())? //Network behaviour ,what bytes and to whom to send on the network
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(u64::MAX))
        }) // Allows us to observe pings indefinitely.
        .build();

    //Now we are all set, we just need to pass a multiaddress to the swarm

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identity by the multiaddress given as the
    // second command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}");
    }

    //Now we can drive the swarm in a loop
    loop {
        //Async stuff, kind of waiting for one of two event to happen, then we print something
        match swarm.select_next_some().await {
            p2::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}")
            }
            p2::swarm::SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
}
