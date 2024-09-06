use std::{collections::HashSet, io::Write, path::PathBuf, time::Duration};

use bytes::Bytes;
use console::style;
use exocore_chain::{
    block::{Block, BlockBuilder, BlockOperations, DataBlock},
    chain::{ChainData, ChainStore},
    operation::{OperationBuilder, OperationFrame, OperationId},
    DirectoryChainStore, DirectoryChainStoreConfig,
};
use exocore_core::{
    cell::{
        Cell, CellConfigExt, CellId, CellNode, CellNodeConfigExt, CellNodeRole, EitherCell,
        FullCell, LocalNode, LocalNodeConfigExt,
    },
    framing::{sized::SizedFrameReaderIterator, FrameReader},
    sec::{auth_token::AuthToken, keys::Keypair},
    time::{Clock, DateTime, Utc},
};
use exocore_protos::{
    core::{cell_node_config, CellConfig, CellNodeConfig, LocalNodeConfig, NodeCellConfig},
    generated::data_chain_capnp::{block_header, chain_operation},
    prost::{Message, ProstTimestampExt},
    reflect::ReflectMessage,
    registry::Registry,
    serde_derive, serde_json,
    store::{Entity, EntityMutation, Trait},
};
use exocore_store::{
    entity::{EntityId, EntityIdRef, TraitId},
    local::ChainEntityIterator,
};

use crate::{app::AppPackage, disco::prompt_discovery_pin, utils::edit_string, Context, *};

#[derive(clap::Parser, Clone)]
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

#[derive(clap::Parser, Clone)]
enum CellCommand {
    /// Initializes a new cell.
    Init(InitOptions),

    /// Lists cells of the node.
    List,

    /// Edit cell configuration.
    Edit,

    /// Join a cell.
    Join(JoinOptions),

    /// Cell nodes related commands.
    Node(NodeOptions),

    /// Prints cell configuration.
    Print(PrintOptions),

    /// Cell apps related commands.
    App(AppOptions),

    /// Cell chain related commands.
    Chain(CellChainOptions),

    /// Generates an auth token.
    GenerateAuthToken(GenerateAuthTokenOptions),
}

#[derive(clap::Parser, Clone)]
struct CellChainOptions {
    #[clap(subcommand)]
    command: CellChainCommand,
}

#[derive(clap::Parser, Clone)]
enum CellChainCommand {
    /// Initializes a chain with a genesis block.
    Init,

    /// Checks the cell's chain integrity.
    Check,

    /// Exports the chain's data.
    Export(ChainExportOptions),

    /// Exports the chain's entities.
    ExportEntities(ChainExportEntitiesOptions),

    /// Imports the chain's data.
    Import(ChainImportOptions),
}

#[derive(clap::Parser, Clone)]
struct InitOptions {
    /// Name of the cell
    #[clap(long)]
    name: Option<String>,

    /// The node will not host the chain locally. The chain will need to be
    /// initialized on another node manually using "create_genesis_block".
    #[clap(long)]
    no_chain: bool,

    /// The node will not host an entity store.
    #[clap(long)]
    no_store: bool,

    /// The node will not host applications.
    #[clap(long)]
    no_app_host: bool,

    /// Don't initialize the chain with a genesis block.
    #[clap(long)]
    no_genesis: bool,
}

/// Cell join related options
#[derive(clap::Parser, Clone)]
struct JoinOptions {
    /// The node will host the chain locally.
    #[clap(long)]
    chain: bool,

    /// The node will host entities store.
    #[clap(long)]
    store: bool,

    /// The node will host applications.
    #[clap(long)]
    app_host: bool,

    /// Manually join a cell using its cell configuration yaml.
    #[clap(long)]
    manual: bool,

    /// Don't unpack applications.
    #[clap(long)]
    no_app_unpack: bool,
}

#[derive(clap::Parser, Clone)]
struct NodeOptions {
    #[clap(subcommand)]
    command: NodeCommand,
}

#[derive(clap::Parser, Clone)]
pub struct PrintOptions {}

#[derive(clap::Parser, Clone)]
enum NodeCommand {
    /// Add a node to the cell.
    Add(NodeAddOptions),

    /// List nodes of the cell.
    List,
}

#[derive(clap::Parser, Clone)]
struct NodeAddOptions {
    /// Manually add the node using its node configuration yaml.
    #[clap(long)]
    manual: bool,
}

#[derive(clap::Parser, Clone)]
struct ChainExportOptions {
    /// File in which chain will be exported.
    file: PathBuf,

    /// Drop index operation deletions used for index garbage collection.
    #[clap(long)]
    drop_operation_deletions: bool,

    /// Export the chain data in a JSON line format.
    #[clap(long)]
    json: bool,

    /// Treat errors as warnings.
    #[clap(long)]
    warning: bool,

    /// Limit export to the given list of entity ids.
    #[clap(long)]
    entities: Vec<String>,
}

#[derive(clap::Parser, Clone)]
struct ChainExportEntitiesOptions {
    /// File in which chain entities will be exported.
    file: PathBuf,
}

#[derive(clap::Parser, Clone)]
struct ChainImportOptions {
    /// Files from which chain will be imported.
    files: Vec<PathBuf>,

    /// Create blocks with a fixed number of operations instead of using
    /// exported block delimiter.
    #[clap(long)]
    operations_per_block: Option<usize>,
}

#[derive(clap::Parser, Clone)]
struct GenerateAuthTokenOptions {
    /// Token expiration duration in days.
    #[clap(long, default_value = "30")]
    expiration_days: u16,
}

#[derive(clap::Parser, Clone)]
struct AppOptions {
    #[clap(subcommand)]
    command: AppCommand,
}

#[derive(clap::Parser, Clone)]
enum AppCommand {
    /// List applications installed in cell.
    List,

    /// Install and unpack an application into the cell.
    Install(AppInstallOptions),

    /// Download and unpacks all applications for which we don't have their
    /// package locally.
    Unpack(AppUnpackOptions),
}

#[derive(clap::Parser, Clone)]
struct AppInstallOptions {
    /// URL to application package to install.
    url: String,

    /// If application already exists, overwrite it.
    #[clap(long)]
    overwrite: bool,
}

#[derive(clap::Parser, Clone, Default)]
struct AppUnpackOptions {
    /// Optional name of the application to unpack.
    /// If none specified, all applications will be unpacked.
    app: Option<String>,

    /// Don't overwrite if application is already installed.
    #[clap(long)]
    no_overwrite: bool,
}

pub async fn handle_cmd(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    match &cell_opts.command {
        CellCommand::Init(init_opts) => cmd_init(ctx, cell_opts, init_opts),
        CellCommand::Node(node_opts) => match &node_opts.command {
            NodeCommand::Add(add_opts) => cmd_node_add(ctx, cell_opts, add_opts).await,
            NodeCommand::List => {
                cmd_node_list(ctx, cell_opts);
                Ok(())
            }
        },
        CellCommand::App(app_opts) => match &app_opts.command {
            AppCommand::List => {
                cmd_app_list(ctx, cell_opts, app_opts);
                Ok(())
            }
            AppCommand::Install(install_opts) => {
                cmd_app_install(ctx, cell_opts, app_opts, install_opts).await
            }
            AppCommand::Unpack(unpack_opts) => {
                cmd_app_unpack(ctx, cell_opts, app_opts, unpack_opts).await
            }
        },
        CellCommand::Join(join_opts) => cmd_join(ctx, cell_opts, join_opts).await,
        CellCommand::List => {
            cmd_list(ctx, cell_opts);
            Ok(())
        }
        CellCommand::Edit => {
            cmd_edit(ctx, cell_opts);
            Ok(())
        }
        CellCommand::Print(opts) => {
            cmd_print(ctx, cell_opts, opts);
            Ok(())
        }
        CellCommand::Chain(chain_opts) => match &chain_opts.command {
            CellChainCommand::Init => cmd_create_genesis_block(ctx, cell_opts),
            CellChainCommand::Check => cmd_check_chain(ctx, cell_opts),
            CellChainCommand::Export(export_opts) => cmd_export_chain(ctx, cell_opts, export_opts),
            CellChainCommand::ExportEntities(export_opts) => {
                cmd_export_chain_entities(ctx, cell_opts, export_opts)
            }
            CellChainCommand::Import(import_opts) => cmd_import_chain(ctx, cell_opts, import_opts),
        },
        CellCommand::GenerateAuthToken(gen_opts) => {
            cmd_generate_auth_token(ctx, cell_opts, gen_opts);
            Ok(())
        }
    }
}

fn cmd_init(
    ctx: &Context,
    _cell_opts: &CellOptions,
    init_opts: &InitOptions,
) -> anyhow::Result<()> {
    let (local_node, _cells) = ctx.options.get_node_and_cells();
    let node_config = local_node.config();

    let cell_keypair = Keypair::generate_ed25519();
    let cell_pk_str = cell_keypair.public().encode_base58_string();

    print_step(format!(
        "Creating new cell in node {}",
        style_value(local_node.name())
    ));

    let cell_name = if let Some(name) = init_opts.name.as_ref() {
        name.clone()
    } else {
        print_spacer();
        dialoguer::Input::with_theme(ctx.dialog_theme.as_ref())
            .with_prompt("Enter the name of the cell")
            .default(cell_keypair.public().generate_name())
            .interact_text()?
    };

    // Create a configuration for the node in the cell
    let cell_node = {
        let mut roles = Vec::new();
        if !init_opts.no_chain {
            print_action(format!(
                "The node will have {} role",
                style_emphasis("chain")
            ));
            roles.push(cell_node_config::Role::ChainRole);
        }

        if !init_opts.no_store {
            print_action(format!(
                "The node will have {} role",
                style_emphasis("store")
            ));
            roles.push(cell_node_config::Role::StoreRole);
        }

        if !init_opts.no_app_host {
            print_action(format!(
                "The node will have {} role",
                style_emphasis("app host")
            ));
            roles.push(cell_node_config::Role::AppHostRole);
        }

        node_config.create_cell_node_config(roles)
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

        let cell_dir = local_node.cell_directory(&cell_id);
        Cell::write_cell_config(&cell_dir, &cell_config)
            .expect("Couldn't write cell configuration");

        cell_config
    };

    add_node_cell(&local_node, &cell_config);

    if !init_opts.no_genesis {
        // Create genesis block
        let (_local_node, either_cells) = ctx.options.get_node_and_cells();

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

    let disco_client = ctx.get_discovery_client();

    let (cell_node, reply_pin, reply_token) = if add_opts.manual {
        let cell_node_ = edit_string(
            "# Paste joining node's public info (result of `exo cell join --manual [--role, [--role, ...]]` on joining node)",
            |config| {
                let config = CellNodeConfig::read_yaml(config.as_bytes())?;
                Ok(config)
            },
        );
        (cell_node_, None, None)
    } else {
        print_spacer();
        let pin = prompt_discovery_pin(ctx, "Enter the joining node discovery PIN");
        let payload = disco_client
            .get(pin)
            .await
            .expect("Couldn't find joining node using the given discovery pin");

        let cell_node_yml = payload
            .decode_payload()
            .expect("Couldn't decode node payload");
        let cell_node = CellNodeConfig::read_yaml(cell_node_yml.as_slice())
            .expect("Couldn't parse joining node config");

        (cell_node, payload.reply_pin, payload.reply_token)
    };

    let node_config = cell_node.node.clone().unwrap();

    let has_role = |role: cell_node_config::Role| -> bool {
        let role_i32: i32 = role.into();
        for another_role in &cell_node.roles {
            if role_i32 == *another_role {
                return true;
            }
        }
        false
    };

    print_step("New node found !");

    print_info(format!("Node name: {}", style_value(&node_config.name)));
    print_info(format!(
        "Public key: {}",
        style_value(&node_config.public_key)
    ));
    print_info(format!(
        "Addresses: {}",
        style_value(node_config.addresses.clone().unwrap_or_default())
    ));

    if !cell_node.roles.is_empty() {
        print_info("Roles:");
        if has_role(cell_node_config::Role::ChainRole) {
            print_action(style_emphasis("chain"));
        }

        if has_role(cell_node_config::Role::StoreRole) {
            print_action(style_emphasis("store"));
        }

        if has_role(cell_node_config::Role::AppHostRole) {
            print_action(style_emphasis("application host"));
        }
    } else {
        print_info("The node will have no roles");
    }

    if !confirm(ctx, "Do you want to add the node to the cell?") {
        return Err(anyhow!("Operation aborted"));
    }

    let mut cell_config = cell.config().clone();
    cell_config.add_node(cell_node);

    print_action("Saving cell config...");
    cell.save_config(&cell_config)
        .expect("Couldn't save cell config");

    let cell_config_yaml = cell_config
        .to_yaml_string()
        .expect("Couldn't convert cell config to yaml");

    if !add_opts.manual {
        disco_client
            .reply(
                reply_pin.expect("Expected reply pin, but didn't find one"),
                reply_token.expect("Expected reply reply, but didn't find one"),
                cell_config_yaml.as_bytes(),
                false,
            )
            .await
            .expect("Couldn't create payload on discovery server");

        print_success(format!(
            "Node {} has been added to the cell",
            style_value(node_config.name)
        ));
    } else if confirm(ctx, "Do you want to print cell configuration?") {
        print!("{}", cell_config_yaml);
    }

    Ok(())
}

fn cmd_node_list(ctx: &Context, cell_opts: &CellOptions) {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    fn print_role(cell_node: &CellNode, role: CellNodeRole) -> String {
        if cell_node.has_role(role) {
            style("✔").green().to_string()
        } else {
            String::new()
        }
    }

    print_spacer();
    let mut rows = Vec::new();
    for cell_node in cell.nodes().iter().all() {
        let node = cell_node.node();
        rows.push(vec![
            node.name().to_string(),
            node.public_key().encode_base58_string(),
            print_role(cell_node, CellNodeRole::Chain),
            print_role(cell_node, CellNodeRole::Store),
            print_role(cell_node, CellNodeRole::AppHost),
        ]);
    }

    print_table(
        vec![
            "Name".to_string(),
            "Public key".to_string(),
            "Chain".to_string(),
            "Store".to_string(),
            "AppHost".to_string(),
        ],
        rows,
    );
}

async fn cmd_join(
    ctx: &Context,
    cell_opts: &CellOptions,
    join_opts: &JoinOptions,
) -> anyhow::Result<()> {
    let (local_node, _) = ctx.options.get_node_and_cells();
    let node_config = local_node.config();

    let mut roles = Vec::new();
    if join_opts.chain {
        print_action(format!(
            "The node will have {} role",
            style_emphasis("chain")
        ));
        roles.push(cell_node_config::Role::ChainRole);
    }

    if join_opts.store {
        print_action(format!(
            "The node will have {} role",
            style_emphasis("store")
        ));
        roles.push(cell_node_config::Role::StoreRole);
    }

    if join_opts.app_host {
        print_action(format!(
            "The node will have {} role",
            style_emphasis("app host")
        ));
        roles.push(cell_node_config::Role::AppHostRole);
    }

    let cell_node = node_config.create_cell_node_config(roles);
    let cell_node_yaml = cell_node
        .to_yaml_string()
        .expect("Couldn't convert cell node config to yaml");

    let cell_config = if !join_opts.manual {
        let disco_client = ctx.get_discovery_client();
        let create_resp = disco_client
            .create(cell_node_yaml.as_bytes(), true)
            .await
            .expect("Couldn't create payload on discovery server");

        print_action(format!(
            "On the host node, enter this discovery pin:\n\n\t\t{}",
            style_value(create_resp.pin.to_formatted_string())
        ));
        print_spacer();

        let payload = disco_client
            .get_loop(create_resp.reply_pin.unwrap(), Duration::from_secs(60))
            .await
            .expect("Couldn't find host node using the given discovery pin");

        let cell_config_yml = payload
            .decode_payload()
            .expect("Couldn't decode cell config payload");
        CellConfig::read_yaml(cell_config_yml.as_slice())
            .expect("Couldn't parse cell config from host node")
    } else {
        print_info("Paste node cell information on host node:");
        println!("{}", cell_node_yaml);

        wait_press_enter();

        edit_string(
            "# Paste config of the cell to join (result of `exo cell print --inline` on host node)",
            |config| {
                let config = CellConfig::read_yaml(config.as_bytes())?;
                Ok(config)
            },
        )
    };

    let cell_id = CellId::from_str(&cell_config.id).expect("Invalid cell id");
    let cell_dir = local_node.cell_directory(&cell_id);

    Cell::write_cell_config(&cell_dir, &cell_config).expect("Couldn't write cell config");

    add_node_cell(&local_node, &cell_config);

    if !join_opts.no_app_unpack {
        let cell_opts = CellOptions {
            public_key: Some(cell_config.public_key.clone()),
            ..cell_opts.clone()
        };
        let (_, cell) = get_cell(ctx, &cell_opts);
        let cell = cell.cell();

        unpack_cell_apps(cell, &AppUnpackOptions::default()).await;
    }

    print_success(format!(
        "Successfully joined cell {} with public key {}",
        style_value(&cell_config.name),
        style_value(&cell_config.public_key),
    ));

    Ok(())
}

fn cmd_edit(ctx: &Context, cell_opts: &CellOptions) {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let cell_config = cell.config();
    let cell_config_yaml = cell_config
        .to_yaml_string()
        .expect("Couldn't convert cell config to yaml");

    let new_cell_config = edit_string(cell_config_yaml, |new_config| {
        let config_bytes = new_config.as_bytes();
        Ok(CellConfig::read_yaml(config_bytes)?)
    });

    cell.save_config(&new_cell_config)
        .expect("Couldn't save cell config");
}

fn cmd_print(ctx: &Context, cell_opts: &CellOptions, _print_opts: &PrintOptions) {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let config_yaml = cell
        .config()
        .to_yaml_string()
        .expect("Couldn't convert cell config to yaml");

    println!("{}", config_yaml,);
}

fn cmd_list(ctx: &Context, _cell_opts: &CellOptions) {
    let (_local_node, either_cells) = ctx.options.get_node_and_cells();

    print_spacer();
    let mut rows = Vec::new();
    for cell in &either_cells {
        rows.push(vec![
            cell.cell().id().to_string(),
            cell.cell().name().to_string(),
            cell.cell().public_key().encode_base58_string(),
        ]);
    }

    print_table(
        vec![
            "ID".to_string(),
            "Name".to_string(),
            "Public key".to_string(),
        ],
        rows,
    );
}

fn cmd_check_chain(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (local_node, cell) = get_cell(ctx, cell_opts);

    let chain_dir = cell
        .cell()
        .chain_directory()
        .as_os_path()
        .expect("Cell is not stored in an OS directory");

    let chain_config = local_node
        .config()
        .chain
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let chain_store = DirectoryChainStore::create_or_open(chain_config.into(), &chain_dir)
        .expect("Couldn't open chain");

    let last_block = chain_store
        .get_last_block()
        .expect("Couldn't get last block of chain")
        .expect("Last block of chain is empty");

    print_spacer();
    let bar = indicatif::ProgressBar::new(last_block.get_height()?);

    let mut prev_block: Option<DataBlock<ChainData>> = None;
    let mut block_count = 0;
    for block in chain_store.blocks_iter(0) {
        let block = block?;

        block_count += 1;
        if let Err(err) = block.validate(prev_block) {
            let block_header_reader = block.header().get_reader();
            let block_height = block_header_reader
                .map(block_header::Reader::get_height)
                .ok();

            bar.finish_and_clear();
            print_error(format!(
                "Block at offset={} height={} is invalid: {}",
                style_value(block.offset()),
                style_value(block_height),
                style_err(err)
            ));
            return Ok(());
        }

        prev_block = Some(block);

        bar.set_position(block_count);
    }

    bar.finish();

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
    let (local_node, cell) = get_cell(ctx, cell_opts);
    let schemas = cell.cell().schemas().as_ref();

    let chain_dir = cell
        .cell()
        .chain_directory()
        .as_os_path()
        .expect("Cell is not stored in an OS directory");

    let chain_config = local_node
        .config()
        .chain
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let chain_store = DirectoryChainStore::create_or_open(chain_config.into(), &chain_dir)
        .expect("Couldn't open chain");

    let mut operation_count = 0;
    let mut warning_count = 0;
    let mut block_count = 0;

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

    let opts: ExportOptions = export_opts.into();

    for block in chain_store.blocks_iter(0) {
        let block = block?;

        block_count += 1;

        if !export_opts.json {
            // create a block proposal to delimitate blocks
            let proposal = OperationBuilder::new_block_proposal_from_data(0, local_node.id(), &[])?;
            let proposal = proposal.sign_and_build(&local_node)?;
            proposal
                .frame
                .copy_to(&mut file_buf)
                .expect("Couldn't write block delimiter");
        }

        let operations = block
            .operations_iter()
            .expect("Couldn't iterate operations from block");
        for operation in operations {
            let res = if export_opts.json {
                export_operation_json(operation, &mut file_buf, schemas, &opts)
            } else {
                export_operation_frame(operation, &mut file_buf, &opts)
            };

            if let Err(err) = res {
                warning_count += 1;
                if !export_opts.warning {
                    panic!("Failed to write operation: {}", err);
                }
            }

            operation_count += 1;
        }

        bar.set_position(block_count);
    }

    file_buf.flush().expect("Couldn't flush file buffer");

    bar.finish_and_clear();

    print_success(format!(
        "Exported {} operations from {} blocks from chain to {} ({} warnings)",
        style_value(operation_count),
        style_value(block_count),
        style_value(&export_opts.file),
        style_value(warning_count),
    ));

    Ok(())
}

struct ExportOptions<'o> {
    opts: &'o ChainExportOptions,
    entities: HashSet<EntityId>,
}

impl<'o> From<&'o ChainExportOptions> for ExportOptions<'o> {
    fn from(opts: &'o ChainExportOptions) -> Self {
        let entities = opts.entities.iter().cloned().collect::<HashSet<_>>();
        ExportOptions { opts, entities }
    }
}

impl<'o> ExportOptions<'o> {
    fn can_export(&self, entity: EntityIdRef) -> bool {
        if self.entities.is_empty() {
            true
        } else {
            self.entities.contains(entity)
        }
    }
}

fn export_operation_frame(
    operation: OperationFrame<&[u8]>,
    out: &mut impl Write,
    opts: &ExportOptions<'_>,
) -> anyhow::Result<()> {
    {
        let reader = operation
            .get_reader()
            .expect("Couldn't get reader on operation");

        // only export entry operations (actual data, not chain maintenance related
        // operations)
        let data = match reader.get_operation().which()? {
            chain_operation::operation::Entry(entry) => entry?.get_data()?,
            _ => return Ok(()),
        };

        let mutation = EntityMutation::decode(data)?;

        if !opts.can_export(&mutation.entity_id) {
            return Ok(());
        }

        // don't keep deleted operations since they were part of the index management
        if opts.opts.drop_operation_deletions {
            use exocore_protos::store::entity_mutation::Mutation;
            match mutation.mutation {
                Some(Mutation::DeleteOperations(_del)) => return Ok(()),
                None => return Ok(()),
                _ => {}
            }
        }
    }

    operation
        .copy_to(out)
        .expect("Couldn't write operation to file buffer");

    Ok(())
}

fn export_operation_json(
    operation: OperationFrame<&[u8]>,
    mut out: &mut impl Write,
    schemas: &Registry,
    opts: &ExportOptions<'_>,
) -> anyhow::Result<()> {
    let reader = operation
        .get_reader()
        .expect("Couldn't get reader on operation");

    // only export entry operations (actual data, not chain maintenance related
    // operations)
    let data = match reader.get_operation().which()? {
        chain_operation::operation::Entry(entry) => entry?.get_data()?,
        _ => return Ok(()),
    };

    let mutation = EntityMutation::decode(data)?;

    if !opts.can_export(&mutation.entity_id) {
        return Ok(());
    }

    use exocore_protos::store::entity_mutation::Mutation;
    match mutation.mutation {
        Some(Mutation::PutTrait(put)) => {
            let trt = put.r#trait.unwrap();
            let msg = trt.message.unwrap();
            let msg_dyn = exocore_protos::reflect::from_prost_any(schemas, &msg)?;
            let msg_json_val = msg_dyn.encode_json(schemas)?;

            let out_obj = serde_json::json!({
                "type": "put_trait",
                "entity_id": mutation.entity_id,
                "trait_id": trt.id,
                "message": msg_json_val,
            });
            serde_json::to_writer(&mut out, &out_obj)?;
        }
        Some(Mutation::DeleteTrait(del)) => {
            let out_obj = serde_json::json!({
                "type": "delete_trait",
                "entity_id": mutation.entity_id,
                "trait_id": del.trait_id,
            });
            serde_json::to_writer(&mut out, &out_obj)?;
        }
        Some(Mutation::DeleteEntity(_del)) => {
            let out_obj = serde_json::json!({
                "type": "delete_entity",
                "entity_id": mutation.entity_id,
            });
            serde_json::to_writer(&mut out, &out_obj)?;
        }
        None | Some(Mutation::DeleteOperations(_)) | Some(Mutation::Test(_)) => {
            return Ok(());
        }
    }

    out.write_all("\n".as_bytes())?;

    Ok(())
}

fn cmd_export_chain_entities(
    ctx: &Context,
    cell_opts: &CellOptions,
    export_opts: &ChainExportEntitiesOptions,
) -> anyhow::Result<()> {
    let (local_node, cell) = get_cell(ctx, cell_opts);
    let schemas = cell.cell().schemas().as_ref();

    let chain_dir = cell
        .cell()
        .chain_directory()
        .as_os_path()
        .expect("Cell is not stored in an OS directory");

    let chain_config = local_node
        .config()
        .chain
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let chain_store = DirectoryChainStore::create_or_open(chain_config.into(), &chain_dir)
        .expect("Couldn't open chain");

    let file = std::fs::File::create(&export_opts.file).expect("Couldn't open exported file");
    let mut file_buf = std::io::BufWriter::new(file);

    print_step(format!(
        "Exporting entities to {}",
        style_value(&export_opts.file)
    ));

    let bar = indicatif::ProgressBar::new_spinner();

    print_step("Sorting mutations...".to_string());
    let entity_iter = ChainEntityIterator::new(&chain_store).unwrap();
    print_step("Mutations sorted, writing entities...".to_string());

    let mut entity_count = 0;
    for (i, entity) in entity_iter.enumerate() {
        let entity = entity.expect("failed to read next entity");

        if entity.deletion_date.is_some() {
            continue;
        }

        let exp = ExportedEntity::from(entity, schemas);
        serde_json::to_writer(&mut file_buf, &exp).unwrap();
        file_buf.write_all("\n".as_bytes()).unwrap();

        entity_count = i;
        bar.set_message(format!("{i}"));
        bar.set_position(i as u64);
    }

    bar.finish_and_clear();

    file_buf.flush().expect("Couldn't flush file buffer");

    print_success(format!("Exported {} entities", style_value(entity_count)));

    Ok(())
}

#[derive(serde_derive::Serialize)]
struct ExportedEntity {
    entity_id: EntityId,
    creation_date: Option<DateTime<Utc>>,
    modification_date: Option<DateTime<Utc>>,
    deletion_date: Option<DateTime<Utc>>,
    traits: Vec<ExportedTrait>,
}

impl ExportedEntity {
    fn from(entity: Entity, schemas: &Registry) -> Self {
        Self {
            entity_id: entity.id,
            creation_date: entity.creation_date.map(|d| d.to_chrono_datetime()),
            modification_date: entity.modification_date.map(|d| d.to_chrono_datetime()),
            deletion_date: entity.deletion_date.map(|d| d.to_chrono_datetime()),
            traits: entity
                .traits
                .into_iter()
                .map(|e| ExportedTrait::from(e, schemas))
                .collect(),
        }
    }
}

#[derive(serde_derive::Serialize)]
struct ExportedTrait {
    trait_id: TraitId,
    creation_date: Option<DateTime<Utc>>,
    modification_date: Option<DateTime<Utc>>,
    deletion_date: Option<DateTime<Utc>>,
    value: serde_json::Value,
}

impl ExportedTrait {
    fn from(trt: Trait, schemas: &Registry) -> Self {
        let msg = trt.message.unwrap();
        let msg_dyn = exocore_protos::reflect::from_prost_any(schemas, &msg).unwrap();
        let msg_json_val = msg_dyn.encode_json(schemas).unwrap();

        Self {
            trait_id: trt.id,
            creation_date: trt.creation_date.map(|d| d.to_chrono_datetime()),
            modification_date: trt.modification_date.map(|d| d.to_chrono_datetime()),
            deletion_date: trt.deletion_date.map(|d| d.to_chrono_datetime()),
            value: msg_json_val,
        }
    }
}

fn cmd_import_chain(
    ctx: &Context,
    cell_opts: &CellOptions,
    import_opts: &ChainImportOptions,
) -> anyhow::Result<()> {
    let (local_node, cell) = get_cell(ctx, cell_opts);
    let full_cell = cell.unwrap_full();

    let clock = Clock::new();
    let node = full_cell.cell().local_node();

    let chain_dir = full_cell
        .cell()
        .chain_directory()
        .as_os_path()
        .expect("Cell is not stored in an OS directory");

    let chain_config = local_node
        .config()
        .chain
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let mut chain_store = DirectoryChainStore::create_or_open(chain_config.into(), &chain_dir)
        .expect("Couldn't open chain");

    if let Some(last_block) = chain_store.get_last_block()? {
        print_info(format!(
            "A chain is already initialized and contains {} blocks.",
            style_value(last_block.get_height()),
        ));

        if confirm(ctx, "Do you want to wipe the chain?") {
            chain_store.truncate_from_offset(0)?;
        } else {
            panic!("Chain is already initialized");
        }
    }

    let genesis_block = exocore_chain::block::BlockBuilder::build_genesis(&full_cell)
        .expect("Couldn't create genesis block");
    chain_store
        .write_block(&genesis_block)
        .expect("Couldn't write genesis block to chain");

    let mut operations_buffer = Vec::new();
    let mut previous_block = genesis_block;
    let mut operations_count = 0;
    let mut blocks_count = 0;

    let mut flush_buffer =
        |block_op_id: OperationId, operations_buffer: &mut Vec<OperationFrame<Bytes>>| {
            if operations_buffer.is_empty() {
                return;
            }

            operations_buffer.sort_by_key(|operation| {
                let operation_reader = operation.get_reader().unwrap();
                operation_reader.get_operation_id()
            });

            let operations = BlockOperations::from_operations(operations_buffer.iter())
                .expect("Couldn't create BlockOperations from operations buffer");

            let block = BlockBuilder::build_with_prev_block(
                full_cell.cell(),
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
        for iter_op_frame in operation_frames_iter {
            let operation_frame =
                exocore_chain::operation::read_operation_frame(iter_op_frame.frame.whole_data())
                    .expect("Couldn't read operation frame");

            let reader = operation_frame
                .get_reader()
                .expect("Couldn't get reader on operation");

            // new block proposal means that previous operations can be flushed
            if reader.get_operation().has_block_propose() {
                if import_opts.operations_per_block.is_none() {
                    let block_op_id = clock.consistent_time(node);
                    flush_buffer(block_op_id.into(), &mut operations_buffer);
                }
                continue;
            }

            operations_count += 1;
            operations_buffer.push(operation_frame.to_owned());

            // if fixed number of operations per block is requested, check if need to be
            // flushed
            if let Some(operations_per_block) = import_opts.operations_per_block {
                if operations_buffer.len() > operations_per_block {
                    let block_op_id = clock.consistent_time(node);
                    flush_buffer(block_op_id.into(), &mut operations_buffer);
                }
            }
        }
    }

    if !operations_buffer.is_empty() {
        let block_op_id = clock.consistent_time(node);
        flush_buffer(block_op_id.into(), &mut operations_buffer);
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
) {
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

    print_info("Token:");
    println!("{}", token.encode_base58_string());
}

fn cmd_create_genesis_block(ctx: &Context, cell_opts: &CellOptions) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let full_cell = cell.unwrap_full();

    create_genesis_block(full_cell)?;

    Ok(())
}

fn cmd_app_list(ctx: &Context, cell_opts: &CellOptions, _app_opts: &AppOptions) {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let cell_apps = cell.applications().get();
    if cell_apps.is_empty() {
        print_warning("No applications installed in cell.");
        return;
    }

    print_spacer();
    let mut rows = Vec::new();
    for cell_app in cell_apps {
        rows.push(vec![
            cell_app.name().to_string(),
            cell_app.version().to_string(),
            cell_app.public_key().encode_base58_string(),
            cell_app.is_loaded().to_string(),
        ]);
    }

    print_table(
        vec![
            "Name".to_string(),
            "Version".to_string(),
            "Public key".to_string(),
            "Installed".to_string(),
        ],
        rows,
    );
}

async fn cmd_app_install(
    ctx: &Context,
    cell_opts: &CellOptions,
    _app_opts: &AppOptions,
    install_opts: &AppInstallOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    let pkg = AppPackage::fetch_package_url(cell, &install_opts.url)
        .await
        .expect("Couldn't fetch app package");
    pkg.install(cell, install_opts.overwrite).await?;

    let manifest = pkg.app.manifest();
    print_success(format!(
        "Application {} version {} got installed into cell.",
        style_value(&manifest.name),
        style_value(&manifest.version),
    ));

    Ok(())
}

async fn cmd_app_unpack(
    ctx: &Context,
    cell_opts: &CellOptions,
    _app_opts: &AppOptions,
    unpack_opts: &AppUnpackOptions,
) -> anyhow::Result<()> {
    let (_, cell) = get_cell(ctx, cell_opts);
    let cell = cell.cell();

    unpack_cell_apps(cell, unpack_opts).await;

    Ok(())
}

async fn unpack_cell_apps(cell: &Cell, unpack_opts: &AppUnpackOptions) {
    for cell_app in cell.applications().get() {
        if let Some(for_app) = &unpack_opts.app {
            if *for_app != cell_app.name() {
                continue;
            }
        }
        if cell_app.is_loaded() {
            // already unpacked
            continue;
        }

        print_info(format!(
            "Installing app {} ({})",
            style_value(cell_app.name()),
            style_value(cell_app.version())
        ));

        let pkg = AppPackage::fetch_package_url(cell, cell_app.package_url())
            .await
            .expect("Couldn't fetch package");

        pkg.install(cell, !unpack_opts.no_overwrite)
            .await
            .expect("Couldn't install app");

        let manifest = pkg.app.manifest();
        print_info(format!(
            "Application {} version {} got installed into cell.",
            style_value(&manifest.name),
            style_value(&manifest.version),
        ));
    }
}

fn get_cell(ctx: &Context, cell_opts: &CellOptions) -> (LocalNode, EitherCell) {
    let (local_node, either_cells) = ctx.options.get_node_and_cells();

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

    (local_node, cell)
}

fn extract_cell_by_pk(either_cells: Vec<EitherCell>, key: &str) -> Option<EitherCell> {
    either_cells
        .into_iter()
        .find(|c| c.cell().public_key().encode_base58_string() == key)
}

fn extract_cell_by_name(either_cells: Vec<EitherCell>, name: &str) -> Option<EitherCell> {
    either_cells.into_iter().find(|c| c.cell().name() == name)
}

fn create_genesis_block(cell: FullCell) -> anyhow::Result<()> {
    let chain_dir = cell
        .cell()
        .chain_directory()
        .as_os_path()
        .expect("Cell is not stored in an OS directory");

    print_step(format!(
        "Creating genesis block for cell {}",
        style_value(cell.cell().public_key().encode_base58_string())
    ));

    std::fs::create_dir_all(&chain_dir)
        .map_err(|err| anyhow!("Couldn't create chain directory: {}", err))?;

    let mut chain_store =
        DirectoryChainStore::create_or_open(DirectoryChainStoreConfig::default(), &chain_dir)
            .map_err(|err| anyhow!("Couldn't create chain store: {}", err))?;
    if chain_store.get_last_block()?.is_some() {
        panic!("Chain is already initialized");
    }

    let genesis_block = exocore_chain::block::BlockBuilder::build_genesis(&cell)
        .map_err(|err| anyhow!("Couldn't create genesis block: {}", err))?;

    chain_store
        .write_block(&genesis_block)
        .map_err(|err| anyhow!("Couldn't write genesis block: {}", err))?;

    Ok(())
}

fn add_node_cell(node: &LocalNode, cell_config: &CellConfig) {
    let node_cell = NodeCellConfig {
        id: cell_config.id.clone(),
        location: None,
    };

    print_action("Writing cell to node");

    let mut node_config = node.config().clone();
    node_config.add_cell(node_cell);

    node.save_config(&node_config)
        .expect("Couldn't write node config");
}

pub fn copy_local_node_to_cells(ctx: &Context, node_config: LocalNodeConfig) {
    let (_local_node, either_cells) = ctx.options.get_node_and_cells();

    for cell in either_cells {
        let mut cell_config = cell.cell().config().clone();
        let changed = if let Some(cell_node_config) = cell_config
            .find_node(&node_config.public_key)
            .and_then(|c| c.node.as_mut())
        {
            cell_node_config.name.clone_from(&node_config.name);
            cell_node_config
                .addresses
                .clone_from(&node_config.addresses);
            true
        } else {
            false
        };

        if changed {
            cell.cell()
                .save_config(&cell_config)
                .expect("Couldn't write cell config");
        }
    }
}
