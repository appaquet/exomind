#![deny(bare_trait_objects)]

extern crate exocore_common;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

#[macro_use]
pub mod lp2p;
pub mod messages;
pub mod transport;

#[cfg(any(test, feature = "tests_utils"))]
pub mod mock;

pub use lp2p::behaviour::{ExocoreBehaviour, ExocoreBehaviourEvent, ExocoreBehaviourMessage};
pub use messages::{InMessage, OutMessage};
pub use transport::Transport;

///
/// Layer of the Exocore architecture to which a message is intented / originating
///
#[derive(Copy, Clone, Debug)]
pub enum TransportLayer {
    Meta = 1,
    Common = 2,
    Data = 3,
}

impl TransportLayer {
    pub fn from_code(code: u8) -> Option<TransportLayer> {
        match code {
            1 => Some(TransportLayer::Meta),
            2 => Some(TransportLayer::Common),
            3 => Some(TransportLayer::Data),
            _ => None,
        }
    }

    pub fn to_code(self) -> u8 {
        self as u8
    }
}

impl Into<u8> for TransportLayer {
    fn into(self) -> u8 {
        self.to_code()
    }
}

///
/// Transport related error
///
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::prelude::*;
    use futures::sync::mpsc;
    use libp2p::core::identity;
    use libp2p::Swarm;
    use libp2p::{Multiaddr, PeerId};
    use tokio::runtime::Runtime;

    use super::*;

    #[test]
    fn test_behaviour() {
        //setup_logging();

        let mut rt = Runtime::new().unwrap();

        let key1 = identity::Keypair::generate_ed25519();
        let peer1 = PeerId::from(key1.public());
        debug!("Peer 1 {}", peer1);
        let transport1 = libp2p::build_development_transport(key1);
        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/3301".parse().unwrap();

        let behaviour1 = ExocoreBehaviour::new();
        let mut swarm1 = libp2p::core::Swarm::new(transport1, behaviour1, peer1.clone());
        Swarm::listen_on(&mut swarm1, addr1.clone()).unwrap();

        let key2 = identity::Keypair::generate_ed25519();
        let peer2 = PeerId::from(key2.public());
        debug!("Peer 2 {}", peer2);
        let transport2 = libp2p::build_development_transport(key2);
        let addr2: Multiaddr = "/ip4/127.0.0.1/tcp/3302".parse().unwrap();

        let behaviour2 = ExocoreBehaviour::new();
        let mut swarm2 = libp2p::core::Swarm::new(transport2, behaviour2, peer2.clone());
        Swarm::listen_on(&mut swarm2, addr2.clone()).unwrap();

        swarm2.add_peer(peer1.clone(), vec![addr1.clone()]);
        swarm1.add_peer(peer2.clone(), vec![addr2.clone()]);

        let (sender1, mut receiver) = mpsc::unbounded::<(PeerId, Vec<u8>)>();
        let mut listening = true;
        rt.spawn(futures::future::poll_fn(move || -> Result<_, ()> {
            while let Async::Ready(Some((peer, data))) =
                receiver.poll().expect("Error polling channel")
            {
                swarm1.send_message(peer, data);
            }

            loop {
                match swarm1.poll().expect("Error while polling swarm") {
                    Async::Ready(Some(data)) => match data {
                        ExocoreBehaviourEvent::Message(msg) => {
                            debug!(
                                "Got message from {}: {}",
                                msg.source,
                                String::from_utf8_lossy(&msg.data)
                            );
                        }
                    },
                    Async::Ready(None) | Async::NotReady => {
                        if !listening {
                            if let Some(a) = Swarm::listeners(&swarm1).next() {
                                debug!("Listening on {:?}", a);
                                listening = true;
                            }
                        }
                        break;
                    }
                }
            }

            Ok(Async::NotReady)
        }));

        let (sender2, mut receiver) = mpsc::unbounded::<(PeerId, Vec<u8>)>();
        let mut listening = true;
        rt.spawn(futures::future::poll_fn(move || -> Result<_, ()> {
            while let Async::Ready(Some((peer, data))) =
                receiver.poll().expect("Error polling channel")
            {
                swarm2.send_message(peer, data);
            }

            loop {
                match swarm2.poll().expect("Error while polling swarm") {
                    Async::Ready(Some(data)) => match data {
                        ExocoreBehaviourEvent::Message(msg) => {
                            debug!(
                                "Got message from {}: {}",
                                msg.source,
                                String::from_utf8_lossy(&msg.data)
                            );
                        }
                    },
                    Async::Ready(None) | Async::NotReady => {
                        if !listening {
                            if let Some(a) = Swarm::listeners(&swarm2).next() {
                                debug!("Listening on {:?}", a);
                                listening = true;
                            }
                        }
                        break;
                    }
                }
            }

            Ok(Async::NotReady)
        }));

        for i in 0..10 {
            std::thread::sleep(Duration::from_millis(200));
            sender1
                .unbounded_send((peer2.clone(), format!("Data for yo #{}", i).into_bytes()))
                .unwrap();
            sender2
                .unbounded_send((peer1.clone(), format!("Data for yo #{}", i).into_bytes()))
                .unwrap();
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
