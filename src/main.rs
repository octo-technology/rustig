#![allow(unused)]

use clap::{Args, Parser, Subcommand};
use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Parser)]
#[clap(name = "rustig")]
#[clap(about = "A simple git-like tool for the terminal", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
    HashObject { path: String },
    CatFile { object: String },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Init {} => {
            init();
        }
        Commands::HashObject { path } => {
            println!("{}", hash_object(path));
        }
        Commands::CatFile { object } => {
            cat_file(object);
        }
    }
}

const GIT_DIR: &str = ".rustig";

fn init() -> std::io::Result<()> {
    fs::create_dir(GIT_DIR).unwrap();
    let current_dir = env::current_dir()?;
    println!(
        "Initialized empty rustig repository in {}/{}",
        current_dir.display(),
        GIT_DIR
    );
    fs::create_dir(format!("{}/objects", GIT_DIR)).unwrap();
    Ok(())
}

fn hash_object(path: String) -> String {
    let mut hasher = Sha1::new();
    let file_content = fs::read(path).expect("Unable to read file");
    hasher.update(&file_content);
    let hash = format!("{:x}", hasher.finalize());
    let mut file = File::create(format!("{}/objects/{}", GIT_DIR, hash)).expect("Unable to create file");
    file.write_all(&file_content);
    return hash;
}

fn cat_file(object: String) {
    let file_content = fs::read(format!("{}/objects/{}", GIT_DIR, object)).expect("Unable to read file");
    println!("{}", String::from_utf8_lossy(&file_content));
}
