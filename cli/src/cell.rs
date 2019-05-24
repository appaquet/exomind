use crate::config::NodeConfig;
use crate::options;
use exocore_common::serialization::framed::TypedFrame;
use exocore_common::serialization::protos::data_chain_capnp::block;
use exocore_data::block::Block;
use exocore_data::chain::ChainStore;
use exocore_data::{DirectoryChainStore, DirectoryChainStoreConfig};

pub fn create_genesis_block(
    _opt: &options::Options,
    cell_opts: &options::CellOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&cell_opts.config)?;
    let local_node = config.create_local_node()?;

    let cell_config = config
        .cells
        .iter()
        .find(|config| config.public_key == cell_opts.public_key)
        .expect("Couldn't find cell with given public key");

    let (full_cell, _cell) = cell_config.create_cell(&local_node)?;
    let full_cell = full_cell.expect("Cannot create genesis block on a non-full cell");

    let mut chain_dir = cell_config.data_directory.clone();
    chain_dir.push("chain");
    std::fs::create_dir_all(&chain_dir)?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_data::block::BlockOwned::new_genesis(&full_cell)?;
    chain_store.write_block(&genesis_block)?;

    Ok(())
}

pub fn check_chain(
    _opt: &options::Options,
    cell_opts: &options::CellOptions,
) -> Result<(), failure::Error> {
    let config = NodeConfig::from_file(&cell_opts.config)?;
    let cell_config = config
        .cells
        .iter()
        .find(|config| config.public_key == cell_opts.public_key)
        .expect("Couldn't find cell with given public key");

    let mut chain_dir = cell_config.data_directory.clone();
    chain_dir.push("chain");
    std::fs::create_dir_all(&chain_dir)?;

    let chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;

    for block in chain_store.blocks_iter(0)? {
        if let Err(err) = block.validate() {
            let block_reader = block.block().get_typed_reader();
            let block_depth = block_reader.map(block::Reader::get_depth).ok();

            error!(
                "Block at offset={} depth={:?} is invalid: {}",
                block.offset(),
                block_depth,
                err
            );
            return Ok(());
        }
    }

    Ok(())
}
