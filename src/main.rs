mod data;

use clap::{Parser, Subcommand};
use data::init;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            init();
        }
    }
}
