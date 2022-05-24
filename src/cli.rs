use crate::data;
use anyhow::Context;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::{env, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(about = "A git clone in Rust", long_about = None)]
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
        Commands::Init {} => init(&context),
        Commands::HashObject { path } => hash_object(&context, path),
        Commands::CatFile { object } => cat_file(&context, object),
    };
}

fn init(context: &data::Context) -> anyhow::Result<()> {
    let repo_dir = context.init()?;
    println!("Initialized empty Rustig repository in {}", repo_dir);
    Ok(())
}

fn hash_object(context: &data::Context, path: PathBuf) -> anyhow::Result<()> {
    let hash = context.hash_object(path, None)?;
    println!("{}", hash);
    Ok(())
}

fn cat_file(context: &data::Context, object: String) -> anyhow::Result<()> {
    let content = context.get_object(object, None)?;
    println!("{}", content);
    Ok(())
}
