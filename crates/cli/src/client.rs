use clap::{Args, Subcommand};
use futures::TryStreamExt;
use unitycatalog_client::UnityCatalogClient;

use crate::GlobalOpts;
use crate::error::Result;
use crate::render::{RenderCtx, ResolvedFormat, render_list, render_one, status};

#[derive(Debug, Args)]
pub struct ClientCommand {
    #[command(subcommand)]
    command: Option<ClientCommands>,
}

#[derive(Debug, Subcommand)]
enum ClientCommands {
    Catalogs(CatalogArgs),
    Schemas(SchemaArgs),
    Tables(TableArgs),
    Volumes(VolumeArgs),
    Functions(FunctionArgs),
}

#[derive(Debug, Args)]
struct CatalogArgs {
    #[command(subcommand)]
    command: Option<CatalogCommands>,
}

#[derive(Debug, Subcommand)]
enum CatalogCommands {
    /// List the catalogs
    List,

    /// Create a new catalog
    Create {
        /// The name of the catalog to create
        #[clap(short, long)]
        name: String,
    },

    /// Get a catalog
    Get {
        /// The name of the catalog to get
        #[clap(short, long)]
        name: String,
    },

    /// Delete a catalog
    Delete {
        /// The name of the catalog to delete
        #[clap(short, long)]
        name: String,

        /// Whether to force delete the catalog
        #[clap(short, long)]
        force: Option<bool>,
    },
}

#[derive(Debug, Args)]
struct SchemaArgs {
    #[command(subcommand)]
    command: Option<SchemaCommands>,
}

#[derive(Debug, Subcommand)]
enum SchemaCommands {
    /// List the schemas in a catalog
    List {
        /// The name of the catalog to list the schemas from
        catalog_name: String,
    },

    /// Create a new schema in a catalog
    Create {
        /// The name of the catalog to create the schema in
        catalog_name: String,
        /// The name of the schema to create
        name: String,
    },

    /// Delete a schema from a catalog
    Delete {
        /// The name of the catalog to delete the schema from
        catalog_name: String,
        /// The name of the schema to delete
        name: String,
    },
}

#[derive(Debug, Args)]
struct TableArgs {
    #[command(subcommand)]
    command: Option<TableCommands>,
}

#[derive(Debug, Subcommand)]
enum TableCommands {
    /// List the tables in a schema
    List {
        /// The name of the parent catalog
        catalog_name: String,
        /// The name of the parent schema
        schema_name: String,
    },

    /// Get a table
    Get {
        /// The name of the parent catalog
        catalog_name: String,
        /// The name of the parent schema
        schema_name: String,
        /// The name of the table to get
        name: String,
    },
}

#[derive(Debug, Args)]
struct VolumeArgs {
    #[command(subcommand)]
    command: Option<VolumeCommands>,
}

#[derive(Debug, Subcommand)]
enum VolumeCommands {
    /// List the volumes in a schema
    List {
        /// The name of the parent catalog
        catalog_name: String,
        /// The name of the parent schema
        schema_name: String,
    },

    /// Get a volume
    Get {
        /// The name of the parent catalog
        catalog_name: String,
        /// The name of the parent schema
        schema_name: String,
        /// The name of the volume to get
        name: String,
    },
}

#[derive(Debug, Args)]
struct FunctionArgs {
    #[command(subcommand)]
    command: Option<FunctionCommands>,
}

#[derive(Debug, Subcommand)]
enum FunctionCommands {
    /// List the functions in a schema
    List {
        /// The name of the parent catalog
        catalog_name: String,
        /// The name of the parent schema
        schema_name: String,
    },
}

pub async fn handle_client(cmd: &ClientCommand, opts: GlobalOpts) -> Result<()> {
    let fmt = opts.output.resolve();
    let ctx = opts.render_ctx();
    let client = opts.client()?;

    match &cmd.command {
        Some(ClientCommands::Catalogs(args)) => handle_catalogs(&client, args, fmt, ctx).await,
        Some(ClientCommands::Schemas(args)) => handle_schemas(&client, args, fmt, ctx).await,
        Some(ClientCommands::Tables(args)) => handle_tables(&client, args, fmt, ctx).await,
        Some(ClientCommands::Volumes(args)) => handle_volumes(&client, args, fmt, ctx).await,
        Some(ClientCommands::Functions(args)) => handle_functions(&client, args, fmt, ctx).await,
        None => {
            status::error("no subcommand provided; see `uc client --help`");
            Ok(())
        }
    }
}

async fn handle_catalogs(
    client: &UnityCatalogClient,
    args: &CatalogArgs,
    fmt: ResolvedFormat,
    ctx: RenderCtx,
) -> Result<()> {
    match &args.command {
        Some(CatalogCommands::List) => {
            let catalogs = client
                .list_catalogs()
                .into_stream()
                .try_collect::<Vec<_>>()
                .await?;
            render_list(&catalogs, fmt, ctx)?;
        }
        Some(CatalogCommands::Create { name }) => {
            let catalog = client.create_catalog(name).await?;
            status::success(&format!("created catalog `{name}`"));
            render_one(&catalog, fmt, ctx)?;
        }
        Some(CatalogCommands::Get { name }) => {
            let catalog = client.catalog(name).get().await?;
            render_one(&catalog, fmt, ctx)?;
        }
        Some(CatalogCommands::Delete { name, force }) => {
            client.catalog(name).delete().with_force(*force).await?;
            status::success(&format!("deleted catalog `{name}`"));
        }
        None => status::error("no subcommand provided; see `uc client catalogs --help`"),
    }
    Ok(())
}

async fn handle_schemas(
    client: &UnityCatalogClient,
    args: &SchemaArgs,
    fmt: ResolvedFormat,
    ctx: RenderCtx,
) -> Result<()> {
    match &args.command {
        Some(SchemaCommands::List { catalog_name }) => {
            let schemas = client
                .catalog(catalog_name)
                .list_schemas()
                .into_stream()
                .try_collect::<Vec<_>>()
                .await?;
            render_list(&schemas, fmt, ctx)?;
        }
        Some(SchemaCommands::Create { catalog_name, name }) => {
            let schema = client.catalog(catalog_name).create_schema(name).await?;
            status::success(&format!("created schema `{catalog_name}.{name}`"));
            render_one(&schema, fmt, ctx)?;
        }
        Some(SchemaCommands::Delete { catalog_name, name }) => {
            client.catalog(catalog_name).schema(name).delete().await?;
            status::success(&format!("deleted schema `{catalog_name}.{name}`"));
        }
        None => status::error("no subcommand provided; see `uc client schemas --help`"),
    }
    Ok(())
}

async fn handle_tables(
    client: &UnityCatalogClient,
    args: &TableArgs,
    fmt: ResolvedFormat,
    ctx: RenderCtx,
) -> Result<()> {
    match &args.command {
        Some(TableCommands::List {
            catalog_name,
            schema_name,
        }) => {
            let tables = client
                .list_tables(catalog_name, schema_name)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await?;
            render_list(&tables, fmt, ctx)?;
        }
        Some(TableCommands::Get {
            catalog_name,
            schema_name,
            name,
        }) => {
            let table = client.table(catalog_name, schema_name, name).get().await?;
            render_one(&table, fmt, ctx)?;
        }
        None => status::error("no subcommand provided; see `uc client tables --help`"),
    }
    Ok(())
}

async fn handle_volumes(
    client: &UnityCatalogClient,
    args: &VolumeArgs,
    fmt: ResolvedFormat,
    ctx: RenderCtx,
) -> Result<()> {
    match &args.command {
        Some(VolumeCommands::List {
            catalog_name,
            schema_name,
        }) => {
            let volumes = client
                .list_volumes(catalog_name, schema_name)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await?;
            render_list(&volumes, fmt, ctx)?;
        }
        Some(VolumeCommands::Get {
            catalog_name,
            schema_name,
            name,
        }) => {
            let volume = client.volume(catalog_name, schema_name, name).get().await?;
            render_one(&volume, fmt, ctx)?;
        }
        None => status::error("no subcommand provided; see `uc client volumes --help`"),
    }
    Ok(())
}

async fn handle_functions(
    client: &UnityCatalogClient,
    args: &FunctionArgs,
    fmt: ResolvedFormat,
    ctx: RenderCtx,
) -> Result<()> {
    match &args.command {
        Some(FunctionCommands::List {
            catalog_name,
            schema_name,
        }) => {
            let functions = client
                .list_functions(catalog_name, schema_name)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await?;
            render_list(&functions, fmt, ctx)?;
        }
        None => status::error("no subcommand provided; see `uc client functions --help`"),
    }
    Ok(())
}
