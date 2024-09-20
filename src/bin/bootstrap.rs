//! Simplied Peer that acts as a DHT bootstrap node.
//! Peers first talk with this peer to enter the peer network
//! so that they can find each other by querying the DHT.

use libp2p::{
    futures::StreamExt,
    identity,
    kad::{self,
          store::MemoryStore,
          Event::*,
          InboundRequest::*,
    },
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId,
    SwarmBuilder,
    
};
use quipu::{
    prelude::*,
    log,
};
use std::{fs::File, io::Read};

#[derive(NetworkBehaviour)]
struct BBehaviour {
    kad: kad::Behaviour<MemoryStore>,
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

    let bbehaviour = BBehaviour { kad };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| {bbehaviour})?
        .build();
    
    swarm.listen_on("/ip4/127.0.0.1/udp/6969/quic-v1".parse()?)?;

    loop {
        tokio::select! {
            swarm_run = swarm.select_next_some() =>
                match swarm_run {
                    SwarmEvent::NewListenAddr {address, ..} => {
                        info!("Listening in {}", address)  
                    },
                    SwarmEvent::ConnectionEstablished {peer_id, ..} => {
                        info!("Connected to peer: {:?}", peer_id)
                    },
                    SwarmEvent::Behaviour(event) => handle_behaviour_event(event)?,
                    e => warn!("Unhandled swarm event: {:?}", e),
                },
        }
    }

}

/// Handle Kademlia DHT events
fn handle_behaviour_event(event: BBehaviourEvent) -> Result<(), Box<dyn Error>> {
    match event {
        BBehaviourEvent::Kad(RoutingUpdated { peer, is_new_peer, addresses, old_peer, .. }) =>
        {
            info!("Kad routing table updated: {:?}, {:?}, {:?}, {:?}", peer, is_new_peer, addresses, old_peer)
        },
        BBehaviourEvent::Kad(ModeChanged { new_mode }) => {
            info!("Kad changed mode into new mode: {:?}", new_mode)
        },
        BBehaviourEvent::Kad(InboundRequest { request }) => {
            match request {
                FindNode { num_closer_peers } => {
                    info!("Found number of closer peers: {}", num_closer_peers)
                },
                GetRecord { num_closer_peers, present_locally } => {
                    info!("Inbound get record request: {:?}, {:?}", num_closer_peers, present_locally)
                },
                PutRecord { source, record, .. } => {
                    info!("Inbound put record request: source {:?}, record {:?}", source, record)
                },
                e => {
                    info!("Unhandled kad inbount request event: {:?}", e)
                }
            }
        },
        BBehaviourEvent::Kad(OutboundQueryProgressed { id, result, stats, ..}) => {
            info!("An outbound query has made progress: {:?} {:?} {:?}", id, result, stats)
        },
        e => info!("Unhandled kad event: {:?}", e),
    }
    Ok(())
}
