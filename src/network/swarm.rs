//! Creation and configuration of a swarm.
//! I'm thinking this module, with the behaviour module
//! makes up the swarm I use to handle the chatroom network events.
//!
//! Three things are needed to create a swarm:
//! 1. network id of this local node in the form of PeerId
//! 2. Implementation of Transport Trait (used to reach other nodes on the network)
//! 3. Implementation of the NetworkBehaviour Trait. (State machine that decides how the
//!    swarm should behave once it is connected to a node.)

use libp2p::{
    identity::Keypair,
    noise,
    // quic,
    swarm,
    tcp,
    yamux,
    mdns,
    PeerId,
    SwarmBuilder,
    swarm::SwarmEvent,
    futures::StreamExt,
};
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
        let peer_keypair = Keypair::generate_ecdsa();
        trace!("\nPublic key: {:#?}\n", peer_keypair.public());

        //Create PeerId from public key
        let peer_id = PeerId::from_public_key(&peer_keypair.public());
        trace!("\nPeerId: {:#?}", peer_id);

        // //Create a quic transport
        // let quic_config = quic::Config::new(&peer_keypair);
        // let mut tcp_transport: tcp::tokio::Transport = tcp::tokio::Transport::default();
        // let mut transport = quic::tokio::Transport::new(quic_config);

        let qbehaviour = QBehaviour::build(peer_id).await?;

        //KEEP WORKING ON THIS BIG BOY
        let swarm = SwarmBuilder::with_existing_identity(peer_keypair.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|_|{qbehaviour})?
            .build();

        Ok(QPeer{peer_id,swarm})
    }

    /// Async functions with loop to handle NetworkBehaviour events.
    pub async fn run_swarm(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            tokio::select! {

                //Deal with swarm events
                swarm_run = self.swarm.select_next_some() =>
                    match swarm_run {
                        SwarmEvent::Behaviour(event) => self.handle_behaviour_event(event)?,

                        SwarmEvent::NewListenAddr { listener_id, address } => {
                            info!("Peer: {:?}, listening on: {:?}", listener_id, address);
                        },
                        SwarmEvent::ConnectionEstablished { peer_id, ..} => {
                            info!("Peer connected to: {:?}", peer_id);
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
            QBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                for(peer_id, addr) in list {
                    info!("Discovered: {:?} with addr {}", peer_id, addr);
                }
            },
            QBehaviourEvent::Mdns(mdns::Event::Expired(list)) => {
                for (peer_id, _multiaddr) in list {
                    info!("mDNS discover peer has expired: {peer_id}");
                    //Here I would remove people from gossipsub for example
                }
            },
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
