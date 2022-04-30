mod data;

use clap::{Parser, Subcommand};
use data::{init, hash_object, cat_object};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize an empty rustig repository
    Init,
    /// Hash a file and return its OID
    #[clap(parse(from_os_str))]
    HashObject { path: std::path::PathBuf },
    /// Print the content of a file by its OID
    CatFile { oid: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => { init() }
        Commands::HashObject { path } => { hash_object(path) }
        Commands::CatFile { oid } => { cat_object(oid) }
    }
}
