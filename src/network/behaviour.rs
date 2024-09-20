//! This module handles the chat network behaviour for libp2p creation of a swarm.
//! The protocol is GossipSub and for peer discovery Kademlia DHT (kad).


use libp2p::{
    kad::{self, store::MemoryStore},
    swarm::NetworkBehaviour,
    PeerId,
};
use crate::prelude::*;

/// Represents the libp2p network behaviour of the application.
/// In fact it derives the libp2p2 swarm::NetworkBehaviour trait.
/// This will be used to create the local peer swarm.
#[derive(NetworkBehaviour)]
pub struct QBehaviour {
    // relay: relay::Behaviour,
    pub kad: kad::Behaviour<MemoryStore>,
}

impl QBehaviour {

    /// Build Qbehaviour
    pub async fn build(local_peer_id: PeerId) -> Result<Self, Box<dyn Error>> {
        
        //Kademlia DHT behaviour
        let kad = kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id));

        let qbehaviour = QBehaviour { kad };

        Ok(qbehaviour)

    }
}

