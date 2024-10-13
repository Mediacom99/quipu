//! This module handles the chat network behaviour for libp2p creation of a swarm.
//! The protocol is GossipSub and for peer discovery Kademlia DHT (kad).

use libp2p::{
    identify,
    identity::PublicKey,
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
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<MemoryStore>,
}

impl QBehaviour {
    /// Build QBehaviour
    pub fn build(public_key: PublicKey, peer_id: PeerId) -> Result<Self, Box<dyn Error>> {
        let identify_config = identify::Config::new(String::new(), public_key);
        let identify = identify::Behaviour::new(identify_config);

        let kad = kad::Behaviour::new(peer_id, MemoryStore::new(peer_id));

        let qbehaviour = QBehaviour { identify, kad };
        Ok(qbehaviour)
    }
}
