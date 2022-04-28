use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about = "A git implementation in Rust", long_about = None)]
struct Cli {
    /// The command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Init a repository
    Init,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init {} => {
            println!("Init!");
        }
    }
}
