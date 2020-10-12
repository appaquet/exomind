use crate::options;
use exocore_chain::block::{Block, BlockOperations, BlockOwned};
use exocore_chain::chain::ChainStore;
use exocore_chain::{
    operation::{OperationFrame, OperationId},
    DirectoryChainStore, DirectoryChainStoreConfig,
};
use exocore_core::{
    cell::{Cell, EitherCell},
    sec::auth_token::AuthToken,
    time::Clock,
};
use exocore_core::{
    framing::{sized::SizedFrameReaderIterator, FrameReader},
    protos::{core::LocalNodeConfig, generated::data_chain_capnp::block_header},
};
use std::{io::Write, time::Duration};

pub fn create_genesis_block(
    _exo_opts: &options::ExoOptions,
    cell_opts: &options::CellOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(cell_opts);
    let full_cell = cell.unwrap_full();

    let chain_dir = full_cell
        .chain_directory()
        .expect("Cell doesn't have a path configured");
    std::fs::create_dir_all(&chain_dir)?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_chain::block::BlockOwned::new_genesis(&full_cell)?;
    chain_store.write_block(&genesis_block)?;

    Ok(())
}

pub fn check_chain(
    _exo_opts: &options::ExoOptions,
    cell_opts: &options::CellOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(cell_opts);

    let chain_dir = cell
        .cell()
        .chain_directory()
        .expect("Cell doesn't have a path configured");
    std::fs::create_dir_all(&chain_dir)?;

    let chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)?;

    let mut block_count = 0;
    for block in chain_store.blocks_iter(0)? {
        block_count += 1;
        if let Err(err) = block.validate() {
            let block_header_reader = block.header().get_reader();
            let block_height = block_header_reader
                .map(block_header::Reader::get_height)
                .ok();

            error!(
                "Block at offset={} height={:?} is invalid: {}",
                block.offset(),
                block_height,
                err
            );
            return Ok(());
        }
    }

    info!("Chain is valid. Analyzed {} blocks.", block_count);

    Ok(())
}

pub fn export_chain(
    _exo_opts: &options::ExoOptions,
    cell_opts: &options::CellOptions,
    export_opts: &options::ChainExportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(cell_opts);

    let chain_dir = cell
        .cell()
        .chain_directory()
        .expect("Cell doesn't have a path configured");
    std::fs::create_dir_all(&chain_dir)?;

    let chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .expect("Couldn't open chain");

    let mut operations_count = 0;
    let mut blocks_count = 0;

    let file = std::fs::File::create(&export_opts.file).expect("Couldn't open exported file");
    let mut file_buf = std::io::BufWriter::new(file);

    for block in chain_store.blocks_iter(0)? {
        blocks_count += 1;

        let operations = block
            .operations_iter()
            .expect("Couldn't iterate operations from block");
        for operation in operations {
            {
                // only export entry operations (actual data, not chain maintenance related
                // operations)
                let reader = operation
                    .get_reader()
                    .expect("Couldn't get reader on operation");
                if !reader.get_operation().has_entry() {
                    continue;
                }
            }

            operations_count += 1;
            operation
                .copy_to(&mut file_buf)
                .expect("Couldn't write operation to file buffer");
        }
    }

    file_buf.flush().expect("Couldn't flush file buffer");

    println!(
        "Exported {} operations from {} blocks from chain",
        operations_count, blocks_count
    );

    Ok(())
}

pub fn import_chain(
    _exo_opts: &options::ExoOptions,
    cell_opts: &options::CellOptions,
    import_opts: &options::ChainImportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(cell_opts);
    let full_cell = cell.unwrap_full();

    let chain_dir = full_cell
        .chain_directory()
        .expect("Cell doesn't have a path configured");
    std::fs::create_dir_all(&chain_dir)?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .expect("Couldn't open chain");
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_chain::block::BlockOwned::new_genesis(&full_cell)
        .expect("Couldn't create genesis block");
    chain_store
        .write_block(&genesis_block)
        .expect("Couldn't write genesis block to chain");

    let mut operations_buffer = Vec::new();
    let mut previous_operation_id = None;
    let mut previous_block = genesis_block;
    let mut operations_count = 0;
    let mut blocks_count = 0;

    let mut flush_buffer =
        |block_op_id: OperationId, operations_buffer: &mut Vec<OperationFrame<Vec<u8>>>| {
            let operations = BlockOperations::from_operations(operations_buffer.iter())
                .expect("Couldn't create BlockOperations from operations buffer");
            let block = BlockOwned::new_with_prev_block(
                &full_cell,
                &previous_block,
                block_op_id,
                operations,
            )
            .expect("Couldn't create new block");
            chain_store
                .write_block(&block)
                .expect("Couldn't write block to chain");

            previous_block = block;
            operations_buffer.clear();
            blocks_count += 1;
        };

    for file in &import_opts.files {
        let file = std::fs::File::open(file).expect("Couldn't open imported file");

        let operation_frames_iter = SizedFrameReaderIterator::new(file);
        for operation_frame in operation_frames_iter {
            let operation =
                exocore_chain::operation::read_operation_frame(operation_frame.frame.whole_data())
                    .expect("Couldn't read operation frame");

            let operation_id = {
                let reader = operation.get_reader()?;
                reader.get_operation_id()
            };

            if let Some(prev_op_id) = previous_operation_id {
                if operation_id < prev_op_id {
                    panic!(
                        "Operations are not sorted! prev={} current={}",
                        prev_op_id, operation_id
                    );
                }
            }
            previous_operation_id = Some(operation_id);

            operations_count += 1;

            operations_buffer.push(operation.to_owned());
            if operations_buffer.len() > import_opts.operations_per_block {
                let block_op_id = operation_id + 1;
                flush_buffer(block_op_id, &mut operations_buffer);
            }
        }
    }

    if !operations_buffer.is_empty() {
        let block_op_id = previous_operation_id.unwrap() + 1;
        flush_buffer(block_op_id, &mut operations_buffer);
    }

    println!(
        "Wrote {} operations in {} blocks to chain",
        operations_count, blocks_count
    );

    Ok(())
}

pub fn generate_auth_token(
    _exo_opts: &options::ExoOptions,
    cell_opts: &options::CellOptions,
    gen_opts: &options::GenerateAuthTokenOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(cell_opts);
    let cell = cell.cell();

    let clock = Clock::new();
    let local_node = cell.local_node();

    let expiration_dur = Duration::from_secs(u64::from(gen_opts.expiration_days) * 86400);
    let expiration = clock.consistent_time(local_node.node()) + expiration_dur;

    let token = AuthToken::new(cell, &clock, Some(expiration)).expect("Couldn't generate token");

    println!("Expiration: {:?}", expiration.to_datetime());
    println!("Token: {}", token.encode_base58_string());

    Ok(())
}

fn get_cell(cell_opts: &options::CellOptions) -> (LocalNodeConfig, EitherCell) {
    let config = exocore_core::cell::node_config_from_yaml_file(&cell_opts.config)
        .expect("Error parsing configuration");
    let (either_cells, _local_node) =
        Cell::new_from_local_node_config(config.clone()).expect("Couldn't create cell from config");

    let cell = if let Some(pk) = &cell_opts.public_key {
        either_cells
            .into_iter()
            .find(|c| c.cell().public_key().encode_base58_string() == *pk)
            .expect("Couldn't find cell with given public key")
    } else {
        if either_cells.len() != 1 {
            panic!("Node config needs to contain only 1 cell if no public key is specified. Use -p option.");
        }

        either_cells.into_iter().next().expect("Couldn't find cell")
    };

    (config, cell)
}
