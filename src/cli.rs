use crate::data::{self, ObjectType, OID};
use anyhow::Context;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::{env, fs, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(about = "A bad git clone, in Rust", long_about = None)]
struct Cli {
    /// The command to run
    #[clap(subcommand)]
    command: Commands,

    #[clap(flatten)]
    verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create an empty rustig repository
    Init,

    /// Compute object ID and create a blob from a file
    #[clap(name = "hash-object", arg_required_else_help = true)]
    HashObject {
        /// File to hash
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
    },

    /// Provide content for repository objects
    #[clap(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        /// Object to show
        #[clap(required = true)]
        object: String,
    },

    /// Create a tree object from the current index
    #[clap(name = "write-tree")]
    WriteTree,

    /// Read tree information into the index
    #[clap(name = "read-tree", arg_required_else_help = true)]
    ReadTree {
        /// Object to read
        #[clap(required = true)]
        object: String,
    },
}

pub fn parse() -> anyhow::Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    log::trace!("Building execution context");
    let work_dir = env::current_dir().context("could not resolve current working directory")?;
    let context = data::Context {
        work_dir: PathBuf::from(&work_dir), // TODO: supplyable via global flag `--work-tree`
        repo_dir: work_dir.join(".rustig"), // TODO: supplyable via global flag `--git-dir`
    };

    return match args.command {
        Commands::Init => init(&context),
        Commands::HashObject { path } => hash_object(&context, path),
        Commands::CatFile { object } => cat_file(&context, object),
        Commands::WriteTree => write_tree(&context),
        Commands::ReadTree { object } => read_tree(&context, object),
    };
}

fn init(context: &data::Context) -> anyhow::Result<()> {
    println!("Initialized empty Rustig repository in {}", context.init()?);
    Ok(())
}

fn hash_object(context: &data::Context, path: PathBuf) -> anyhow::Result<()> {
    context.ensure_init()?;
    let data = fs::read(&path).context(format!("could not read '{}'", path.display()))?;
    println!("{}", context.hash_object(data, ObjectType::Blob)?);
    Ok(())
}

fn cat_file(context: &data::Context, object: String) -> anyhow::Result<()> {
    context.ensure_init()?;
    let data = context.get_object(OID(object), &[ObjectType::Blob])?;
    println!("{}", String::from_utf8_lossy(&data));
    Ok(())
}

fn write_tree(context: &data::Context) -> anyhow::Result<()> {
    context.ensure_init()?;
    println!("{}", context.write_tree(&context.work_dir)?);
    Ok(())
}

fn read_tree(context: &data::Context, object: String) -> anyhow::Result<()> {
    context.ensure_init()?;
    context.read_tree(OID(object), &context.work_dir)
}
