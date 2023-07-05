use crate::data::{self, ObjectType, OID};
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(about = "A bad git clone, in Rust", long_about = None)]
struct Cli {
    /// The command to run
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: Verbosity,

    #[command(flatten)]
    flags: Flags,
}

#[derive(Args, Debug)]
pub struct Flags {
    /// Set the path to the working tree
    #[arg(long, required = false, global = true, default_value = ".")]
    work_tree: PathBuf,

    /// Set the path to the repository
    #[arg(long, required = false, global = true, default_value = ".rustig.db")]
    repo_file: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an empty rustig repository
    Init,

    /// Compute object ID and create a blob from a file
    #[command(name = "hash-object", arg_required_else_help = true)]
    HashObject {
        /// File to hash
        #[arg(required = true)]
        path: PathBuf,
    },

    /// Provide content for repository objects
    #[command(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        /// Object to show
        #[arg(required = true)]
        object: String,
    },

    /// Create a tree object from the current index
    #[command(name = "write-tree")]
    WriteTree,

    /// Read tree information into the index
    #[command(name = "read-tree", arg_required_else_help = true)]
    ReadTree {
        /// Object to read
        #[arg(required = true)]
        object: String,
    },
}

pub async fn parse() -> anyhow::Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match args.command {
        Commands::Init => init(args.flags).await,
        Commands::HashObject { path } => hash_object(args.flags, path).await,
        Commands::CatFile { object } => cat_file(args.flags, object).await,
        Commands::WriteTree => write_tree(args.flags).await,
        Commands::ReadTree { object } => read_tree(args.flags, object).await,
    }
}

async fn init(flags: Flags) -> anyhow::Result<()> {
    data::Context::new(flags.repo_file.clone(), true).await?;
    println!(
        "Initialized empty Rustig repository in {}",
        flags.repo_file.display()
    );
    Ok(())
}

async fn hash_object(flags: Flags, path: PathBuf) -> anyhow::Result<()> {
    let mut ctx = data::Context::new(flags.repo_file, false).await?;
    let data = fs::read(&path).context(format!("could not read '{}'", path.display()))?;
    println!("{}", ctx.hash_object(data, ObjectType::Blob).await?);
    Ok(())
}

async fn cat_file(flags: Flags, object: String) -> anyhow::Result<()> {
    let mut ctx = data::Context::new(flags.repo_file, false).await?;
    let data = ctx.get_object(&OID(object), &[ObjectType::Blob]).await?;
    println!("{}", String::from_utf8_lossy(&data));
    Ok(())
}

async fn write_tree(flags: Flags) -> anyhow::Result<()> {
    let mut ctx = data::Context::new(flags.repo_file, false).await?;
    println!("{}", ctx.write_tree(&flags.work_tree).await?);
    Ok(())
}

async fn read_tree(flags: Flags, object: String) -> anyhow::Result<()> {
    let mut ctx = data::Context::new(flags.repo_file, false).await?;
    ctx.read_tree(OID(object), &flags.work_tree).await
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
