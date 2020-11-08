use crate::{
    utils::{edit_file, edit_string, shell_prompt},
    Options,
};
use clap::Clap;
use exocore_chain::block::{Block, BlockOperations, BlockOwned};
use exocore_chain::chain::ChainStore;
use exocore_chain::{
    operation::{OperationFrame, OperationId},
    DirectoryChainStore, DirectoryChainStoreConfig,
};
use exocore_core::{
    cell::CellConfigExt,
    cell::CellId,
    cell::FullCell,
    cell::LocalNode,
    cell::LocalNodeConfigExt,
    cell::NodeConfigExt,
    cell::{Cell, EitherCell},
    protos::core::cell_node_config,
    protos::core::node_cell_config,
    protos::core::CellConfig,
    protos::core::CellNodeConfig,
    protos::core::NodeCellConfig,
    protos::core::NodeConfig,
    sec::auth_token::AuthToken,
    sec::keys::Keypair,
    time::Clock,
};
use exocore_core::{
    framing::{sized::SizedFrameReaderIterator, FrameReader},
    protos::{core::LocalNodeConfig, generated::data_chain_capnp::block_header},
};
use std::{io::Write, path::PathBuf, time::Duration};

#[derive(Clap)]
pub struct CellOptions {
    /// Public key of the cell we want to make an action on. If not specified
    /// and the node config only contains 1 cell, this cell will be taken.
    #[clap(long, short)]
    public_key: Option<String>,

    /// Name of the cell we want to make an action on. If not specified
    /// and the node config only contains 1 cell, this cell will be taken.
    #[clap(long, short)]
    name: Option<String>,

    #[clap(subcommand)]
    command: CellCommand,
}

#[derive(Clap)]
enum CellCommand {
    /// Initializes a new cell.
    Init(InitOptions),

    /// Lists cells of the node.
    List,

    /// Edit cell configuration.
    Edit,

    /// Join a cell.
    Join(JoinOptions),

    /// Cell nodes options.
    Node(NodeOptions),

    /// Print cell configuration.
    Print(PrintOptions),

    /// Check the cell's chain integrity.
    CheckChain,

    /// Export the chain's data.
    Exportchain(ChainExportOptions),

    /// Import the chain's data.
    ImportChain(ChainImportOptions),

    /// Generate an auth token.
    GenerateAuthToken(GenerateAuthTokenOptions),

    /// Create genesis block of the chain.
    CreateGenesisBlock,
}

/// Cell intialization related options
#[derive(Clap)]
struct InitOptions {
    /// Name of the cell
    #[clap(long)]
    name: Option<String>,

    /// The node will not host the chain locally. The chain will need to be
    /// initialized on another node manually using "create_genesis_block".
    #[clap(long)]
    no_chain: bool,

    /// The node will not expose an entity store server.
    #[clap(long)]
    no_store: bool,

    /// Don't create genesis block.
    #[clap(long)]
    no_genesis: bool,
}

/// Cell join related options
#[derive(Clap)]
struct JoinOptions {}

#[derive(Clap)]
struct NodeOptions {
    #[clap(subcommand)]
    command: NodeCommand,
}

#[derive(Clap)]
pub struct PrintOptions {
    /// Inline configuration instead of pointing to external objects.
    #[clap(long)]
    pub inline: bool,
}

#[derive(Clap)]
enum NodeCommand {
    /// Add a node to the cell.
    Add(NodeAddOptions),
}

#[derive(Clap)]
struct NodeAddOptions {
    /// The node will host the chain locally.
    #[clap(long)]
    chain: bool,

    /// The node will host entities store.
    #[clap(long)]
    store: bool,
}

#[derive(Clap)]
struct ChainExportOptions {
    // File in which chain will be exported.
    file: PathBuf,
}

#[derive(Clap)]
struct ChainImportOptions {
    // Number of operations per blocks.
    #[clap(long, default_value = "30")]
    operations_per_block: usize,

    // Files from which chain will be imported.
    files: Vec<PathBuf>,
}

#[derive(Clap)]
struct GenerateAuthTokenOptions {
    // Token expiration duration in days.
    #[clap(long, default_value = "30")]
    expiration_days: u16,
}

pub fn handle_cmd(exo_opts: &Options, cell_opts: &CellOptions) -> anyhow::Result<()> {
    match &cell_opts.command {
        CellCommand::Init(init_opts) => cmd_init(&exo_opts, cell_opts, init_opts),
        CellCommand::Node(node_opts) => match &node_opts.command {
            NodeCommand::Add(add_opts) => cmd_node_add(exo_opts, cell_opts, add_opts),
        },
        CellCommand::Join(join_opts) => cmd_join(exo_opts, cell_opts, join_opts),
        CellCommand::List => cmd_list(&exo_opts, cell_opts),
        CellCommand::Edit => cmd_edit(&exo_opts, cell_opts),
        CellCommand::Print(opts) => cmd_print(&exo_opts, cell_opts, opts),
        CellCommand::CheckChain => cmd_check_chain(&exo_opts, cell_opts),
        CellCommand::Exportchain(export_opts) => {
            cmd_export_chain(&exo_opts, cell_opts, export_opts)
        }
        CellCommand::ImportChain(import_opts) => {
            cmd_import_chain(&exo_opts, cell_opts, import_opts)
        }
        CellCommand::GenerateAuthToken(gen_opts) => {
            cmd_generate_auth_token(&exo_opts, cell_opts, gen_opts)
        }
        CellCommand::CreateGenesisBlock => cmd_create_genesis_block(&exo_opts, cell_opts),
    }
}

fn cmd_init(
    exo_opts: &Options,
    _cell_opts: &CellOptions,
    init_opts: &InitOptions,
) -> anyhow::Result<()> {
    let node_config = exo_opts.read_configuration();
    let node = LocalNode::new_from_config(node_config.clone())
        .expect("Couldn't create node from node config");

    let mut cell_name = node.name().to_string();
    if init_opts.name.is_none() {
        let resp = shell_prompt("Cell name", None)?;
        if let Some(resp) = resp {
            cell_name = resp;
        }
    }

    let cell_node = {
        // Create a configuration for the node in the cell
        let mut cell_node = CellNodeConfig {
            node: Some(NodeConfig {
                public_key: node.public_key().encode_base58_string(),
                id: node.id().to_string(),
                name: node.name().to_string(),
                addresses: node_config.addresses.clone(),
            }),
            ..Default::default()
        };

        if !init_opts.no_chain {
            cell_node
                .roles
                .push(cell_node_config::Role::ChainRole.into());
        }

        if !init_opts.no_store {
            cell_node
                .roles
                .push(cell_node_config::Role::StoreRole.into());
        }

        cell_node
    };

    let cell_config = {
        // Create & write cell configuration
        let cell_keypair = Keypair::generate_ed25519();
        let cell_pk_str = cell_keypair.public().encode_base58_string();

        let cell_id = CellId::from_public_key(&cell_keypair.public());
        let cell_config = CellConfig {
            keypair: cell_keypair.encode_base58_string(),
            public_key: cell_pk_str,
            id: cell_id.to_string(),
            name: cell_name,
            nodes: vec![cell_node],
            ..Default::default()
        };

        write_cell_config(exo_opts, &cell_config);

        cell_config
    };

    // Write node configuration with new cell
    add_node_config_cell(exo_opts, &node_config, &cell_config);

    if !init_opts.no_genesis {
        // Create genesis block
        let node_config = exo_opts.read_configuration();
        let (either_cells, _local_node) = Cell::new_from_local_node_config(node_config)
            .expect("Couldn't create cell from config");

        let cell = extract_cell_by_pk(either_cells, &cell_config.public_key)
            .expect("Couldn't find just created cell in config");

        let full_cell = cell.unwrap_full();

        create_genesis_block(full_cell).expect("Couldn't create genesis block");
    }

    Ok(())
}

fn cmd_node_add(
    exo_opts: &Options,
    cell_opts: &CellOptions,
    add_opts: &NodeAddOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    let mut cell_config =
        CellConfig::from_yaml_file(&config_path).expect("Couldn't read cell config");

    let node_config = edit_string(
        "# Paste joining node's public info (result of `exo config print --cell` on joining node)",
        |config| {
            let config = NodeConfig::from_yaml(config.as_bytes())?;
            Ok(config)
        },
    );

    let mut cell_node = CellNodeConfig {
        node: Some(node_config),
        roles: vec![],
    };

    if add_opts.chain {
        cell_node
            .roles
            .push(cell_node_config::Role::ChainRole.into());
    }

    if add_opts.store {
        cell_node
            .roles
            .push(cell_node_config::Role::StoreRole.into());
    }

    cell_config.nodes.push(cell_node);

    cell_config
        .to_yaml_file(&config_path)
        .expect("Couldn't write cell config");

    Ok(())
}

fn cmd_join(
    exo_opts: &Options,
    _cell_opts: &CellOptions,
    _join_opts: &JoinOptions,
) -> anyhow::Result<()> {
    let node_config = exo_opts.read_configuration();

    let cell_config = edit_string(
        "# Paste config of the cell to join (result of `exo cell print` on host node)",
        |config| {
            let config = CellConfig::from_yaml(config.as_bytes())?;
            Ok(config)
        },
    );

    write_cell_config(exo_opts, &cell_config);

    add_node_config_cell(exo_opts, &node_config, &cell_config);

    Ok(())
}

fn cmd_edit(exo_opts: &Options, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    edit_file(&config_path, |temp_path| {
        CellConfig::from_yaml_file(temp_path)?;
        Ok(())
    });

    Ok(())
}

fn cmd_print(
    exo_opts: &Options,
    cell_opts: &CellOptions,
    print_opts: &PrintOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    let mut config = CellConfig::from_yaml_file(config_path).expect("Coudlnt' read cell config");

    if print_opts.inline {
        config = config.inlined().expect("Couldn't inline config");
    }

    println!(
        "{}",
        config
            .to_yaml()
            .expect("Couldn't convert cell config to yaml")
    );

    Ok(())
}

fn cmd_list(exo_opts: &Options, _cell_opts: &CellOptions) -> anyhow::Result<()> {
    let config = exo_opts.read_configuration();
    let (either_cells, _local_node) =
        Cell::new_from_local_node_config(config).expect("Couldn't create cell from config");

    for cell in &either_cells {
        println!(
            "Name: {} Public key: {}",
            cell.cell().name(),
            cell.cell().public_key().encode_base58_string()
        );
    }

    Ok(())
}

fn cmd_check_chain(exo_opts: &Options, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);

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

fn cmd_export_chain(
    exo_opts: &Options,
    cell_opts: &CellOptions,
    export_opts: &ChainExportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);

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

fn cmd_import_chain(
    exo_opts: &Options,
    cell_opts: &CellOptions,
    import_opts: &ChainImportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
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

fn cmd_generate_auth_token(
    exo_opts: &Options,
    cell_opts: &CellOptions,
    gen_opts: &GenerateAuthTokenOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
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

fn cmd_create_genesis_block(exo_opts: &Options, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(exo_opts, cell_opts);
    let full_cell = cell.unwrap_full();

    create_genesis_block(full_cell)?;

    Ok(())
}

fn get_cell(exo_opts: &Options, cell_opts: &CellOptions) -> (LocalNodeConfig, EitherCell) {
    let config = exo_opts.read_configuration();
    let (either_cells, _local_node) =
        Cell::new_from_local_node_config(config.clone()).expect("Couldn't create cell from config");

    let cell = if let Some(pk) = &cell_opts.public_key {
        extract_cell_by_pk(either_cells, pk.as_str())
            .expect("Couldn't find cell with given public key")
    } else if let Some(name) = &cell_opts.name {
        extract_cell_by_name(either_cells, name.as_str())
            .expect("Couldn't find cell with given name")
    } else {
        if either_cells.len() != 1 {
            panic!("Node config needs to contain only 1 cell if no public key is specified. Use -p option.");
        }

        either_cells.into_iter().next().expect("Couldn't find cell")
    };

    (config, cell)
}

fn extract_cell_by_pk(either_cells: Vec<EitherCell>, key: &str) -> Option<EitherCell> {
    either_cells
        .into_iter()
        .find(|c| c.cell().public_key().encode_base58_string() == key)
}

fn extract_cell_by_name(either_cells: Vec<EitherCell>, name: &str) -> Option<EitherCell> {
    either_cells.into_iter().find(|c| c.cell().name() == name)
}

fn cell_config_path(cell: &Cell) -> PathBuf {
    let cell_directory = cell.cell_directory().expect("Couldn't find cell directory");
    cell_directory.join("cell.yaml")
}

fn create_genesis_block(cell: FullCell) -> anyhow::Result<()> {
    let chain_dir = cell
        .chain_directory()
        .expect("Couldn't find chain directory");

    std::fs::create_dir_all(&chain_dir)
        .map_err(|err| anyhow!("Couldn't create chain directory: {}", err))?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .map_err(|err| anyhow!("Couldn't create chain store: {}", err))?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_chain::block::BlockOwned::new_genesis(&cell)
        .map_err(|err| anyhow!("Couldn't create genesis block: {}", err))?;

    chain_store
        .write_block(&genesis_block)
        .map_err(|err| anyhow!("Couldn't write genesis block: {}", err))?;

    Ok(())
}

fn write_cell_config(exo_opts: &Options, config: &CellConfig) {
    if config.public_key.is_empty() {
        panic!("Expected cell to have a public key");
    }

    let mut cell_dir = exo_opts.dir_path();
    cell_dir.push("cells");
    cell_dir.push(config.public_key.clone());

    std::fs::create_dir_all(&cell_dir).expect("Couldn't create cell directory");

    let cell_config_path = cell_dir.join("cell.yaml");
    config
        .to_yaml_file(cell_config_path)
        .expect("Couldn't write cell config");
}

fn add_node_config_cell(
    exo_opts: &Options,
    node_config: &LocalNodeConfig,
    cell_config: &CellConfig,
) {
    let node_cell = NodeCellConfig {
        location: Some(node_cell_config::Location::Path(format!(
            "cells/{}",
            &cell_config.public_key
        ))),
    };

    let mut node_config = node_config.clone();
    node_config.cells.push(node_cell);
    node_config
        .to_yaml_file(exo_opts.conf_path())
        .expect("Couldn't write node config");
}
