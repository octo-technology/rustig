#![allow(unused)]

use clap::{Args, Parser, Subcommand};
use std::fs;
use std::env;

#[derive(Debug,Parser)]
#[clap(name = "rustig")]
#[clap(about = "A simple git-like tool for the terminal", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Init {} => {
            init();
        }
    }
}

const GIR_DIR: &str = ".rustig";

fn init() -> std::io::Result<()> {
   fs::create_dir(GIR_DIR).unwrap();
   let current_dir = env::current_dir()?;
   println!("Initialized empty rustig repository in {}/{}",current_dir.display(), GIR_DIR);
   Ok(())
}
