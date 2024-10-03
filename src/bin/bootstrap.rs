//! Simplied Peer that acts as a DHT bootstrap node.
//! Peers first talk with this peer to enter the peer network
//! so that they can find each other by querying the DHT.

use libp2p::{
    futures::StreamExt, identify, identity, kad::{self, store::{MemoryStore, RecordStore}, Event::*, InboundRequest::*
    }, swarm::{NetworkBehaviour, SwarmEvent, dial_opts::DialOpts}, PeerId, StreamProtocol, Swarm, SwarmBuilder
    
};
use quipu::{
    prelude::*,
    log,
};

use tokio::io::{AsyncBufReadExt, BufReader};

use std::{fs::File, str::FromStr};
use std::io::Read;

#[derive(NetworkBehaviour)]
struct BBehaviour {
    kad: kad::Behaviour<MemoryStore>,
    identify: identify::Behaviour,
}

fn load_keypair_from_file() -> Result<identity::Keypair, Box<dyn Error>> {
    // Open the keypair file
    let mut file = File::open("bootstrap_keypair.bin")?;
    
    // Read the keypair bytes from the file
    let mut keypair_bytes = Vec::new();
    file.read_to_end(&mut keypair_bytes)?;
    
    // Decode the keypair from bytes
    let keypair = identity::Keypair::from_protobuf_encoding(&keypair_bytes)?;
    
    Ok(keypair)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::tracing_subscriber_setup("info").await;
    info!("quipu is starting...");

    let local_key = load_keypair_from_file()?;
    
    let local_peer_id = PeerId::from_public_key(&local_key.public());

    let kad = kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id));

    let identify_config = identify::Config::new(String::new(), local_key.public());
    let identify = identify::Behaviour::new(identify_config);

    let bbehaviour = BBehaviour { kad, identify};

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| {bbehaviour})?
        .build();
    
    swarm.listen_on("/ip4/127.0.0.1/udp/16969/quic-v1".parse()?)?;

    let mut reader = BufReader::new(tokio::io::stdin());

    loop {
        let mut buffer = String::new();
        tokio::select! {

            // Handle user input
            num_bytes_read = reader.read_line(&mut buffer) => {
                trace!("Read {} bytes from stdin", num_bytes_read?);
                parse_cli(&mut swarm, buffer).await.unwrap_or_else(|e| {
                    warn!("cli parsing error: {}", e);
                });
            },
            swarm_run = swarm.select_next_some() =>
                match swarm_run {                    
                    SwarmEvent::Behaviour(event) => handle_behaviour_event(event, &mut swarm)?,
                    e => warn!("Unhandled swarm event: {:?}", e),
                },
        }
    }

}

/// Handle Kademlia DHT events
fn handle_behaviour_event(event: BBehaviourEvent, swarm: &mut Swarm<BBehaviour>) -> Result<(), Box<dyn Error>> {
    match event {
        BBehaviourEvent::Identify(event) => {
            match event {
                identify::Event::Received { connection_id, peer_id, info } => {
                    info!("New Peer identified:\n\tPeerId: {:?}\n\tWarn: {:#?}", peer_id, info);
                },
                e => warn!("Unhandled identify event: {:?}", e),
            }
        },
        e => warn!("Unhandled behaviour event: {:?}", e),
    }
    Ok(())
}


// Handling user input
// TODO there is definitely a better way to do this
// TODO also fix the help message
use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(
    no_binary_name = true,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    DialPeer(DialPeerArgs),
}

#[derive(Args, Debug)]
struct DialPeerArgs {
    #[arg(required = true)]
    peer_id: String,
} 

async fn parse_cli(swarm: &mut Swarm<BBehaviour>, line: String) -> Result<(), Box<dyn Error>>{

    let args: Vec<&str> = line
        .split_whitespace()
        .collect();
    
    if args.is_empty() {
        warn!("Empty user command");
    } else {
        match Cli::try_parse_from(args)?.command {
            Some(Commands::DialPeer(args)) => {
                println!("PeerId entered: {}", args.peer_id);
                let peer_id = PeerId::from_str(&args.peer_id)?;
                let dial_opt = DialOpts::peer_id(peer_id)
                    .addresses(vec!["/ip4/127.0.0.1/udp/16969/quic-v1".parse().unwrap()])
                    .build();
                swarm.dial(dial_opt)?;                
            },
            None => {
                warn!("Unrecognized command");
            }
        }
    }
    Ok(())

    
}
