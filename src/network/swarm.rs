//! Creation and configuration of a swarm.
//! I'm thinking this module, with the behaviour module
//! makes up the swarm I use to handle the chatroom network events.
//!
//! Three things are needed to create a swarm:
//! 1. network id of this local node in the form of PeerId
//! 2. Implementation of Transport Trait (used to reach other nodes on the network)
//! 3. Implementation of the NetworkBehaviour Trait. (State machine that decides how the
//!    swarm should behave once it is connected to a node.)
//!
//!
//! The idea for peer discovery:
//! 1. add bootstrap node and bootstrap kademlia
//! 2. (set as provider of own peerID as key) put record into kad peer_id -> username
//! 3. you can lookup a certain peerID in the network and receive its listening addresses
//!    through identify events.
//! If this does not work I can store the listening addresses as the value with peerID as key.

use super::behaviour::{QBehaviour, QBehaviourEvent};
use crate::prelude::*;
use libp2p::{
    futures::StreamExt, identify, identity::Keypair, kad, swarm::{self, SwarmEvent}, PeerId, SwarmBuilder
};
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Local Quipu Peer identified by the PeerId.
/// containts the libp2p swarm.
/// wil contain stuff for data persistence.
pub struct QPeer {
    pub swarm: swarm::Swarm<QBehaviour>,
}

impl QPeer {
    #[instrument]
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        //TODO Choose which encryption method
        let keypair = Keypair::generate_ed25519();
        debug!("\nPublic key: {:#?}\n", keypair.public());

        //Create PeerId from public key
        let peer_id = PeerId::from_public_key(&keypair.public());
        info!("\nLocal PeerId: {:#?}", peer_id);

        let behaviour = QBehaviour::build(keypair.public(), peer_id)?;

        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic() // quic as transport
            .with_behaviour(|_| behaviour)? // custom network behaviour
            .build();
        Ok(QPeer { swarm })
    }

    /// Async functions with loop to handle NetworkBehaviour events.
    pub async fn run_swarm(&mut self) -> Result<(), Box<dyn Error>> {
        // Listen on localhost and on OS assigned port
        self.swarm
            .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse()?)?;

        // Add DHT bootstrap address
        let bootstrap_address = "/ip4/127.0.0.1/udp/16969/quic-v1".parse()?;
        let bootstrap_id =
            PeerId::from_str("12D3KooWHZyoJBiNub6zYfSgh4SktsVCjxxXChfASVqGfc8YvtPt")?;
        self.swarm
            .behaviour_mut()
            .kad
            .add_address(&bootstrap_id, bootstrap_address);

        // async user input buffer reader
        let mut reader = BufReader::new(tokio::io::stdin());

        loop {
            let mut buffer = String::new();
            tokio::select! {

                // Handle user input
                // TODO should handle this in a different tokio task
                _ = reader.read_line(&mut buffer) => {
                    parse_cli(&mut self.swarm, buffer).await?;
                },

                // Deal with swarm events
                swarm_event = self.swarm.select_next_some() =>
                    match swarm_event {
                        // Handle all behaviour events
                        SwarmEvent::Behaviour(behav_event) => {
                            match behav_event {
                                QBehaviourEvent::Identify(id_ev) =>
                                    self.handle_identify_event(id_ev)?,
                                QBehaviourEvent::Kad(kad_ev) =>
                                    self.handle_kad_event(kad_ev)?,
                            }
                        },
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Local listening address: {}", address.to_string());
                        },
                        e => debug!("Unhandled swarm event: {:?}", e),
                    },

                // Check for shutdown signal
                // TODO should implement proper tokio shutdown
                exit_signal = tokio::signal::ctrl_c() =>
                    match exit_signal {
                        Ok(())  => {
                            std::process::exit(1);
                        },
                        Err(err) => {
                            error!("Unable to listen for ctrl_c shutdown signal: {}", err);
                        }
                    }
            }
        }
    }

    /// Handles Identify events
    fn handle_identify_event(
        &mut self,
        identify_event: identify::Event,
    ) -> Result<(), Box<dyn Error>> {
        match identify_event {
            // On receving info from an established connection
            // to a listener we add the listener to the kad
            // routing table.
            identify::Event::Received { peer_id, info, .. } => {
                todo!();
            },
            e => debug!("Unhandled identify event: {:?}", e),
        }
        Ok(())
    }

    /// Handles Kademlia events
    fn handle_kad_event(&mut self, kad_event: kad::Event) -> Result<(), Box<dyn Error>> {
        match kad_event {
            // The local routing table has been updated
            kad::Event::RoutingUpdated {
                peer,
                is_new_peer, // new peer (t) or an existing peer updated their address (f)
                addresses,
                ..
            } => {
                info!("Routing updated ({}): {}", is_new_peer, peer.to_base58());
            },
            kad::Event::OutboundQueryProgressed { result, .. } => {
                todo!();
            },
            e => warn!("Unhandled kad event: {:?}", e),
        }
        Ok(())
    }
}

/// Handles user input in a very bad way
// TODO Should really make this safer and better I guess
async fn parse_cli(
    swarm: &mut swarm::Swarm<QBehaviour>,
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
