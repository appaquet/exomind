use crate::options;
use exocore_core::cell::{Node, NodeId};
use exocore_core::sec::keys::Keypair;

pub fn cmd_generate(
    _exo_opts: &options::ExoOptions,
    _keys_opts: &options::KeysOptions,
) -> anyhow::Result<()> {
    let keypair = Keypair::generate_ed25519();
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
