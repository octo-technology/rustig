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
        type_: String
    },

    /// Provide content or type and size information for repository objects
    #[clap(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        /// The name of the object to show.
        #[clap(required = true)]
        object: String,
        #[clap(default_value_t=String::from("blob"),short, long)]
        expected: String,
    },
}

pub fn parse() -> io::Result<()> {
    let args: Cli = Cli::parse();

    return match args.command {
        Commands::Init {} => init(),
        Commands::HashObject { path, type_ } => hash_object(path, type_),
        Commands::CatFile { object, expected } => cat_file(object, expected),
    };
}

fn init() -> io::Result<()> {
    let git_dir:String = data::init()?;
    println!("Initialized empty Rustig repository in {}", git_dir);
    Ok(())
}

fn hash_object(path: PathBuf, type_: String) -> io::Result<()> {
    let hash:String = data::hash_object(path, type_)?;
    println!("{}", hash);
    Ok(())
}

fn cat_file(object: String, expected: String) -> io::Result<()> {
    let content:String = data::cat_file(object, expected)?;
    println!("{}", content);
    Ok(())
}
