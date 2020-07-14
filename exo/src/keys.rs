use crate::options;
use exocore_core::cell::{Node, NodeId};
use exocore_core::crypto::keys::Keypair;

pub fn generate(_opt: &options::Options, keys_opts: &options::KeysOptions) -> anyhow::Result<()> {
    let keypair = match keys_opts.algorithm {
        options::KeyAlgorithm::Ed25519 => Keypair::generate_ed25519(),
        options::KeyAlgorithm::Rsa => unimplemented!(),
    };

    println!("keypair: {}", keypair.encode_base58_string());
    println!("public_key: {}", keypair.public().encode_base58_string());
    println!(
        "name: {}",
        Node::new_from_public_key(keypair.public()).name()
    );

    let node_id = NodeId::from_public_key(&keypair.public());
    println!("id: {}", node_id);

    Ok(())
}
