//! Simplied Peer that acts as a DHT bootstrap node.
//! Peers first talk with this peer to enter the peer network
//! so that they can find each other by querying the DHT.

use libp2p::{
    futures::StreamExt,
    identify, identity,
    kad::{
        self,
        store::{MemoryStore, RecordStore},
    },
    swarm::{self, NetworkBehaviour, SwarmEvent},
    PeerId, Swarm, SwarmBuilder,
    core::ConnectedPoint,
};
use quipu::{log, prelude::*};

use tokio::io::{AsyncBufReadExt, BufReader};

use std::fs::File;
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

    let bbehaviour = BBehaviour { kad, identify };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| bbehaviour)?
        .build();

    // Set kad behaviour to operate ONLY in server mode
    // swarm.behaviour_mut().kad.set_mode(Some(kad::Mode::Server));

    swarm.listen_on("/ip4/127.0.0.1/udp/16969/quic-v1".parse()?)?;

    let mut reader = BufReader::new(tokio::io::stdin());

    loop {
        let mut buffer = String::new();
        tokio::select! {

            // Handle user input
            num_bytes_read = reader.read_line(&mut buffer) => {
                todo!();
            },

            // Handle swarm events
            swarm_run = swarm.select_next_some() =>
                match swarm_run {
                    SwarmEvent::Behaviour(event) => handle_behaviour_event(event, &mut swarm)?,
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, ..} => {
                        todo!();
                    },
                    e => info!("Unhandled swarm event: {:?}", e),
                },
        }
    }
}

/// Handle Kademlia DHT events
fn handle_behaviour_event(
    event: BBehaviourEvent,
    swarm: &mut Swarm<BBehaviour>,
) -> Result<(), Box<dyn Error>> {
    match event {
        BBehaviourEvent::Identify(event) => {
            match event {
                identify::Event::Received { peer_id, info, .. } => {
                    todo!();
                }
                e => debug!("Unhandled identify event: {:?}", e),
            }
        }
        BBehaviourEvent::Kad(event) => match event {
            kad::Event::RoutingUpdated {
                peer,
                is_new_peer,
                addresses,
                ..
            } => {
                info!("Routing updated ({}): {}", is_new_peer, peer.to_base58());
            }
            e => warn!("Unhandled kad event: {:?}", e),
        },
    }
    Ok(())
}


/// Handles user input in a very bad way
// TODO Should really make this safer and better I guess
async fn parse_cli(
    swarm: &mut swarm::Swarm<BBehaviour>,
    line: String,
) -> Result<(), Box<dyn Error>> {
    if line.matches("get-closest-peers").next().is_some() {
        todo!();
    } else {
        println!("Available commands:");
        println!("\thelp                             \t\t show this help message");
        Ok(())
    }
}
