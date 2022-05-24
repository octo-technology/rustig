use crate::data;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use std::{env, io, path::PathBuf};

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

pub fn parse() -> io::Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    log::trace!("Building execution context");
    let context = data::Context {
        work_dir: env::current_dir()?, // TODO: supplyable via global flag `--work-tree`
        repo_dir: env::current_dir()?.join(".rustig"), // TODO: supplyable via global flag `--git-dir`
    };

    return match args.command {
        Commands::Init {} => init(&context),
        Commands::HashObject { path } => hash_object(&context, path),
        Commands::CatFile { object } => cat_file(&context, object),
    };
}

fn init(context: &data::Context) -> io::Result<()> {
    let repo_dir = context.init()?;
    println!("Initialized empty Rustig repository in {}", repo_dir);
    Ok(())
}

fn hash_object(context: &data::Context, path: PathBuf) -> io::Result<()> {
    let hash = context.hash_object(path)?;
    println!("{}", hash);
    Ok(())
}

fn cat_file(context: &data::Context, object: String) -> io::Result<()> {
    let content = context.get_object(object)?;
    println!("{}", content);
    Ok(())
}
