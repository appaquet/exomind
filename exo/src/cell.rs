use crate::{
    disco::prompt_discovery_pin,
    term::*,
    utils::{edit_file, edit_string},
    Context,
};
use clap::Clap;
use exocore_chain::{
    block::{Block, BlockOperations, BlockOwned},
    chain::ChainStore,
    operation::{OperationFrame, OperationId},
    DirectoryChainStore, DirectoryChainStoreConfig,
};
use exocore_core::{
    cell::{
        Cell, CellConfigExt, CellId, EitherCell, FullCell, LocalNode, LocalNodeConfigExt,
        NodeConfigExt,
    },
    framing::{sized::SizedFrameReaderIterator, FrameReader},
    protos::{
        core::{
            cell_node_config, node_cell_config, CellConfig, CellNodeConfig, LocalNodeConfig,
            NodeCellConfig, NodeConfig,
        },
        generated::data_chain_capnp::block_header,
    },
    sec::{auth_token::AuthToken, keys::Keypair},
    time::Clock,
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
    /// Initialize a new cell.
    Init(InitOptions),

    /// List cells of the node.
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
    ExportChain(ChainExportOptions),

    /// Import the chain's data.
    ImportChain(ChainImportOptions),

    /// Generate an auth token.
    GenerateAuthToken(GenerateAuthTokenOptions),

    /// Create genesis block of the chain.
    CreateGenesisBlock,
}

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
struct JoinOptions {
    /// Manually join a cell using its cell configuration yaml.
    #[clap(long)]
    manual: bool,
}

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

    /// Manually add the node using its node configuration yaml.
    #[clap(long)]
    manual: bool,
}

#[derive(Clap)]
struct ChainExportOptions {
    // File in which chain will be exported.
    file: PathBuf,
}

#[derive(Clap)]
struct ChainImportOptions {
    // Number of operations per block.
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

pub async fn handle_cmd(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    match &cell_opts.command {
        CellCommand::Init(init_opts) => cmd_init(ctx, cell_opts, init_opts),
        CellCommand::Node(node_opts) => match &node_opts.command {
            NodeCommand::Add(add_opts) => cmd_node_add(ctx, cell_opts, add_opts).await,
        },
        CellCommand::Join(join_opts) => cmd_join(ctx, cell_opts, join_opts).await,
        CellCommand::List => cmd_list(ctx, cell_opts),
        CellCommand::Edit => cmd_edit(ctx, cell_opts),
        CellCommand::Print(opts) => cmd_print(ctx, cell_opts, opts),
        CellCommand::CheckChain => cmd_check_chain(ctx, cell_opts),
        CellCommand::ExportChain(export_opts) => cmd_export_chain(ctx, cell_opts, export_opts),
        CellCommand::ImportChain(import_opts) => cmd_import_chain(ctx, cell_opts, import_opts),
        CellCommand::GenerateAuthToken(gen_opts) => {
            cmd_generate_auth_token(ctx, cell_opts, gen_opts)
        }
        CellCommand::CreateGenesisBlock => cmd_create_genesis_block(ctx, cell_opts),
    }
}

fn cmd_init(
    ctx: &Context,
    _cell_opts: &CellOptions,
    init_opts: &InitOptions,
) -> anyhow::Result<()> {
    let node_config = ctx.options.read_configuration();
    let node = LocalNode::new_from_config(node_config.clone())
        .expect("Couldn't create node from node config");

    let cell_keypair = Keypair::generate_ed25519();
    let cell_pk_str = cell_keypair.public().encode_base58_string();

    print_step(format!(
        "Creating new cell in node {}",
        style_value(node.name())
    ));

    let mut cell_name = node.name().to_string();
    if init_opts.name.is_none() {
        print_spacer();
        cell_name = dialoguer::Input::with_theme(ctx.dialog_theme.as_ref())
            .with_prompt("Enter the name of the cell")
            .default(cell_keypair.public().generate_name())
            .interact_text()?;
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
            print_action(format!(
                "The node will have {} role",
                style_emphasis("chain")
            ));
            cell_node
                .roles
                .push(cell_node_config::Role::ChainRole.into());
        }

        if !init_opts.no_store {
            print_action(format!(
                "The node will have {} role",
                style_emphasis("store")
            ));
            cell_node
                .roles
                .push(cell_node_config::Role::StoreRole.into());
        }

        cell_node
    };

    let cell_config = {
        // Create & write cell configuration
        let cell_id = CellId::from_public_key(&cell_keypair.public());
        let cell_config = CellConfig {
            keypair: cell_keypair.encode_base58_string(),
            public_key: cell_pk_str,
            id: cell_id.to_string(),
            name: cell_name,
            nodes: vec![cell_node],
            ..Default::default()
        };

        write_cell_config(ctx, &cell_config);

        cell_config
    };

    add_node_config_cell(ctx, &node_config, &cell_config);

    if !init_opts.no_genesis {
        // Create genesis block
        let node_config = ctx.options.read_configuration();
        let (either_cells, _local_node) = Cell::new_from_local_node_config(node_config)
            .expect("Couldn't create cell from config");

        let cell = extract_cell_by_pk(either_cells, &cell_config.public_key)
            .expect("Couldn't find just created cell in config");

        let full_cell = cell.unwrap_full();

        create_genesis_block(full_cell).expect("Couldn't create genesis block");
    }

    print_success(format!(
        "Created cell named {} with public key {}",
        style_value(cell_config.name),
        style_value(cell_config.public_key),
    ));

    Ok(())
}

async fn cmd_node_add(
    ctx: &Context,
    cell_opts: &CellOptions,
    add_opts: &NodeAddOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    let mut cell_config =
        CellConfig::from_yaml_file(&config_path).expect("Couldn't read cell config");

    let disco_client = ctx.get_discovery_client();

    let node_config = if add_opts.manual {
        edit_string(
            "# Paste joining node's public info (result of `exo config print --cell` on joining node)",
            |config| {
                let config = NodeConfig::from_yaml(config.as_bytes())?;
                Ok(config)
            },
        )
    } else {
        print_spacer();
        let pin = prompt_discovery_pin(ctx, "Enter the joining node discovery PIN");
        let node_config = disco_client
            .get(pin)
            .await
            .expect("Couldn't find joining node using the given discovery pin");

        NodeConfig::from_yaml(node_config.as_slice()).expect("Couldn't parse joining node config")
    };

    let mut cell_node = CellNodeConfig {
        node: Some(node_config),
        roles: vec![],
    };

    if add_opts.chain {
        print_action(format!(
            "The node will have {} role",
            style_emphasis("chain")
        ));
        cell_node
            .roles
            .push(cell_node_config::Role::ChainRole.into());
    }

    if add_opts.store {
        print_action(format!(
            "The node will have {} role",
            style_emphasis("store")
        ));
        cell_node
            .roles
            .push(cell_node_config::Role::StoreRole.into());
    }

    // TODO: Should replace if already exists
    cell_config.nodes.push(cell_node);

    print_action(format!(
        "Writing cell config to {}",
        style_value(&config_path)
    ));
    cell_config
        .to_yaml_file(&config_path)
        .expect("Couldn't write cell config");

    if !add_opts.manual {
        let cell_config_inlined = cell_config
            .inlined()
            .expect("Couldn't inline cell config")
            .to_yaml()
            .expect("Couldn't convert cell config to yaml");

        let create_resp = disco_client
            .create(cell_config_inlined.as_bytes())
            .await
            .expect("Couldn't create payload on discovery server");

        print_action(format!(
            "On the joining node, enter this discovery pin:\n\n\t\t{}",
            style_value(create_resp.id.to_formatted_string())
        ));
    }

    Ok(())
}

async fn cmd_join(
    ctx: &Context,
    _cell_opts: &CellOptions,
    join_opts: &JoinOptions,
) -> anyhow::Result<()> {
    let node_config = ctx.options.read_configuration();
    let disco_client = ctx.get_discovery_client();

    let cell_node = NodeConfig {
        id: node_config.id.clone(),
        name: node_config.name.clone(),
        public_key: node_config.public_key.clone(),
        addresses: node_config.addresses.clone(),
    };
    let cell_node_yaml = cell_node
        .to_yaml()
        .expect("Couldn't convert cell node config to yaml");

    let cell_config = if !join_opts.manual {
        let create_resp = disco_client
            .create(cell_node_yaml.as_bytes())
            .await
            .expect("Couldn't create payload on discovery server");

        print_action(format!(
            "On the host node, enter this discovery pin:\n\n\t\t{}",
            style_value(create_resp.id.to_formatted_string())
        ));

        print_spacer();
        let pin = prompt_discovery_pin(ctx, "Enter the host discovery PIN");
        let cell_config = disco_client
            .get(pin)
            .await
            .expect("Couldn't find host node using the given discovery pin");

        CellConfig::from_yaml(cell_config.as_slice())
            .expect("Couldn't parse cell config from host node")
    } else {
        edit_string(
            "# Paste config of the cell to join (result of `exo cell print --inline` on host node)",
            |config| {
                let config = CellConfig::from_yaml(config.as_bytes())?;
                Ok(config)
            },
        )
    };

    write_cell_config(ctx, &cell_config);

    add_node_config_cell(ctx, &node_config, &cell_config);

    print_success(format!(
        "Successfully joined cell {} with public key {}",
        style_value(&cell_config.name),
        style_value(&cell_config.public_key),
    ));

    Ok(())
}

fn cmd_edit(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    edit_file(&config_path, |temp_path| {
        CellConfig::from_yaml_file(temp_path)?;
        Ok(())
    });

    Ok(())
}

fn cmd_print(
    ctx: &Context,
    cell_opts: &CellOptions,
    print_opts: &PrintOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let config_path = cell_config_path(cell);
    let mut config = CellConfig::from_yaml_file(config_path).expect("Couldn't read cell config");

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

fn cmd_list(ctx: &Context, _cell_opts: &CellOptions) -> anyhow::Result<()> {
    let config = ctx.options.read_configuration();
    let (either_cells, _local_node) =
        Cell::new_from_local_node_config(config).expect("Couldn't create cell from config");

    print_spacer();
    let mut rows = Vec::new();
    for cell in &either_cells {
        rows.push(vec![
            cell.cell().name().to_string(),
            cell.cell().public_key().encode_base58_string(),
        ]);
    }

    print_table(vec!["Name".to_string(), "Public key".to_string()], rows);

    Ok(())
}

fn cmd_check_chain(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);

    let chain_dir = cell
        .cell()
        .chain_directory()
        .expect("Cell doesn't have a path configured");

    print_spacer();
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

            print_error(format!(
                "Block at offset={} height={} is invalid: {}",
                style_value(block.offset()),
                style_value(block_height),
                style_err(err)
            ));
            return Ok(());
        }
    }

    print_success(format!(
        "Chain is valid. Analyzed {} blocks.",
        style_value(block_count)
    ));

    Ok(())
}

fn cmd_export_chain(
    ctx: &Context,
    cell_opts: &CellOptions,
    export_opts: &ChainExportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);

    let chain_dir = cell
        .cell()
        .chain_directory()
        .expect("Cell doesn't have a path configured");

    let chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .expect("Couldn't open chain");

    let mut operations_count = 0;
    let mut blocks_count = 0;

    let file = std::fs::File::create(&export_opts.file).expect("Couldn't open exported file");
    let mut file_buf = std::io::BufWriter::new(file);

    print_step(format!(
        "Exporting chain to {}",
        style_value(&export_opts.file)
    ));

    let last_block = chain_store
        .get_last_block()
        .expect("Couldn't get last block of chain")
        .expect("Last block of chain is empty");

    print_spacer();
    let bar = indicatif::ProgressBar::new(last_block.get_height()?);

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

            bar.set_position(blocks_count);
        }
    }

    file_buf.flush().expect("Couldn't flush file buffer");

    bar.finish();
    print_success(format!(
        "Exported {} operations from {} blocks from chain to {}",
        style_value(operations_count),
        style_value(blocks_count),
        style_value(&export_opts.file)
    ));

    Ok(())
}

fn cmd_import_chain(
    ctx: &Context,
    cell_opts: &CellOptions,
    import_opts: &ChainImportOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let full_cell = cell.unwrap_full();

    let chain_dir = full_cell
        .chain_directory()
        .expect("Cell doesn't have a path configured");

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .expect("Couldn't open chain");
    if let Some(last_block) = chain_store.get_last_block()? {
        print_info(format!(
            "A chain is already initialized and contains {} blocks.",
            style_value(last_block.get_height()),
        ));

        print_spacer();
        let over = dialoguer::Confirm::with_theme(ctx.dialog_theme.as_ref())
            .with_prompt("Do you want to wipe the chain?")
            .interact()
            .expect("Couldn't get prompt answer");

        if over {
            chain_store.truncate_from_offset(0)?;
        } else {
            panic!("Chain is already initialized");
        }
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
        print_step(format!("Importing file {}", style_value(file)));
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

    print_success(format!(
        "Wrote {} operations in {} blocks to chain",
        style_value(operations_count),
        style_value(blocks_count)
    ));

    Ok(())
}

fn cmd_generate_auth_token(
    ctx: &Context,
    cell_opts: &CellOptions,
    gen_opts: &GenerateAuthTokenOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let clock = Clock::new();
    let local_node = cell.local_node();

    let expiration_dur = Duration::from_secs(u64::from(gen_opts.expiration_days) * 86400);
    let expiration = clock.consistent_time(local_node.node()) + expiration_dur;

    let token = AuthToken::new(cell, &clock, Some(expiration)).expect("Couldn't generate token");

    print_info(format!(
        "Expiration: {}",
        style_value(expiration.to_datetime())
    ));
    print_info(format!(
        "Token: {}",
        style_value(token.encode_base58_string())
    ));

    Ok(())
}

fn cmd_create_genesis_block(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let full_cell = cell.unwrap_full();

    create_genesis_block(full_cell)?;

    Ok(())
}

fn get_cell(ctx: &Context, cell_opts: &CellOptions) -> (LocalNodeConfig, EitherCell) {
    let config = ctx.options.read_configuration();
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

    print_info(format!(
        "Using cell {} with public key {}",
        style_value(cell.cell().name()),
        style_value(cell.cell().public_key().encode_base58_string())
    ));

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

    print_step(format!(
        "Creating genesis block for cell {}",
        style_value(cell.public_key().encode_base58_string())
    ));

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

fn write_cell_config(ctx: &Context, config: &CellConfig) {
    if config.public_key.is_empty() {
        panic!("Expected cell to have a public key");
    }

    let mut cell_dir = ctx.options.dir_path();
    cell_dir.push("cells");
    cell_dir.push(config.public_key.clone());

    print_action(format!(
        "Creating cell directory {}",
        style_value(&cell_dir)
    ));
    std::fs::create_dir_all(&cell_dir).expect("Couldn't create cell directory");

    let cell_config_path = cell_dir.join("cell.yaml");
    print_action(format!(
        "Writing cell config to {}",
        style_value(&cell_config_path)
    ));
    config
        .to_yaml_file(cell_config_path)
        .expect("Couldn't write cell config");
}

fn add_node_config_cell(ctx: &Context, node_config: &LocalNodeConfig, cell_config: &CellConfig) {
    let node_cell = NodeCellConfig {
        location: Some(node_cell_config::Location::Path(format!(
            "cells/{}",
            &cell_config.public_key
        ))),
    };

    print_action(format!(
        "Writing cell to node config {}",
        style_value(ctx.options.conf_path())
    ));

    let mut node_config = node_config.clone();
    node_config.add_cell(node_cell);

    node_config
        .to_yaml_file(ctx.options.conf_path())
        .expect("Couldn't write node config");
}
