//! Creation and configuration of a swarm.
//! I'm thinking this module, with the behaviour module
//! makes up the swarm I use to handle the chatroom network events.
//!
//! Three things are needed to create a swarm:
//! 1. network id of this local node in the form of PeerId
//! 2. Implementation of Transport Trait (used to reach other nodes on the network)
//! 3. Implementation of the NetworkBehaviour Trait. (State machine that decides how the
//!    swarm should behave once it is connected to a node.)

use std::str::{FromStr, Lines};
use libp2p::{
    futures::StreamExt,
    identity::Keypair,
    noise,
    swarm::{self, SwarmEvent},
    tcp,
    yamux,
    PeerId,
    SwarmBuilder
};
use tokio::io::{BufReader, AsyncBufReadExt};
use crate::prelude::*;
use super::behaviour::{ QBehaviour, QBehaviourEvent};

/// Local Quipu Peer identified by the PeerId
/// containts the libp2p swarm
pub struct QPeer {
    pub peer_id: PeerId,
    pub swarm: swarm::Swarm<QBehaviour>,
}

impl QPeer {
    
    #[instrument]
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        //TODO Choose which encryption method
        //Generate new public-private key-pair using ECDSA
        let keypair = Keypair::generate_ed25519();
        trace!("\nPublic key: {:#?}\n", keypair.public());

        //Create PeerId from public key
        let peer_id = PeerId::from_public_key(&keypair.public());
        trace!("\nPeerId: {:#?}", peer_id);

        //Create a quic transport
        // let quic_config = quic::Config::new(&peer_keypair);
        // let mut tcp_transport: tcp::tokio::Transport = tcp::tokio::Transport::default();
        // let mut transport = quic::tokio::Transport::new(quic_config);

        let behaviour = QBehaviour::build(peer_id).await?;

        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|_|{behaviour})?
            .build();
        
        Ok(QPeer{peer_id, swarm})
    }

    /// Async functions with loop to handle NetworkBehaviour events.
    pub async fn run_swarm(&mut self) -> Result<(), Box<dyn Error>> {

        // Listen all interfaces and on OS assigned port
        self.swarm.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse()?)?;
        // self.swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

        //Add DHT bootstrap address
        let bootstrap_address = "/ip4/127.0.0.1/udp/6969/quic-v1".parse()?;
        let bootstrap_id = PeerId::from_str("12D3KooWHZyoJBiNub6zYfSgh4SktsVCjxxXChfASVqGfc8YvtPt")?;
        self.swarm.behaviour_mut().kad.add_address(&bootstrap_id, bootstrap_address);

        let mut reader = BufReader::new(tokio::io::stdin());

        
        loop {
            let mut buffer = String::new();
            tokio::select! {

                //Deal with command line interface
                bytes_read = reader.read_line(&mut buffer) => {
                    debug!("Read {} bytes from stdin", bytes_read?);
                    println!("Command entered: {}", buffer);
                    if buffer.starts_with("findpeer") {
                        println!("Asked for command findpeer");
                        self.swarm.behaviour_mut().kad.get_closest_peers(self.peer_id);
                    } else {
                        warn!("This commmand was not recognized");
                    }
                },
                
                //Deal with swarm events
                swarm_run = self.swarm.select_next_some() =>
                    match swarm_run {
                        SwarmEvent::Behaviour(event) => self.handle_behaviour_event(event)?,
                        
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Peer listening on: {}", address);
                        },
                        SwarmEvent::ConnectionEstablished { peer_id, ..} => {
                            info!("Peer connected to: {}", peer_id.to_base58());
                        },
                        e => warn!("Unhandled swarm event: {:?}", e),
                    },
                
                //Check for shutdown signal
                exit_signal = tokio::signal::ctrl_c() =>
                    match exit_signal {
                        Ok(())  => {
                            //FIXME implement tokio graceful shutdown
                            std::process::exit(1);
                        },
                        Err(err) => {
                            error!("Unable to listen for ctrl_c shutdown signal: {}", err);
                        }
                    }
            }
        }
    }

    /// Handles events related to this custom NetworkBehaviour
    fn handle_behaviour_event(&mut self, event: QBehaviourEvent) -> Result<(), Box<dyn Error>> {
        match event {
            e => warn!("Unhandled network behaviour event: {:?}", e),
         }
        Ok(())
    }

    
}




///Test for ChatMessage serialization/deserialization using bincode
#[tokio::test]
async fn qpeer_init() {
    crate::log::tracing_subscriber_setup("trace").await;
    let _peer1 = QPeer::init().await.unwrap();
}
