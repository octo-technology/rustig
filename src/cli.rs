use crate::data;
use clap::{Parser, Subcommand};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(about = "A git implementation in Rust", long_about = None)]
struct Cli {
    /// The command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create an empty Rustig repository or reinitialize an existing one
    Init,

    /// Compute object ID and optionally creates a blob from a file
    #[clap(name = "hash-object", arg_required_else_help = true)]
    HashObject {
        /// Object to hash
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,

        /// Specify the type
        #[clap(short = 't', default_value_t=String::from("blob"))]
        type_: String,
    },

    /// Provide content or type and size information for repository objects
    #[clap(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        // /// Type of obq
        // type_: String,
        /// The name of the object to show.
        #[clap(required = true, last = true)]
        object: String,
    },
}

pub fn parse() -> io::Result<()> {
    let args = Cli::parse();

    return match args.command {
        Commands::Init {} => init(),
        Commands::HashObject { path, type_: _ } => hash_object(path),
        Commands::CatFile { object } => cat_file(object),
    };
}

fn init() -> io::Result<()> {
    let git_dir = data::init()?;
    println!("Initialized empty Rustig repository in {}", git_dir);
    Ok(())
}

fn hash_object(path: PathBuf) -> io::Result<()> {
    let hash = data::hash_object(path)?;
    println!("{}", hash);
    Ok(())
}

fn cat_file(object: String) -> io::Result<()> {
    let content = data::cat_file(object)?;
    println!("{}", content);
    Ok(())
}
