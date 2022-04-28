use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use sha1_smol::Digest;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::{env, fs};

lazy_static! {
    static ref GIT_DIR: PathBuf = env::current_dir().unwrap().join(".rustig");
}

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

    /// Hash object
    #[clap(name = "hash-object", arg_required_else_help = true)]
    HashObject {
        /// Stuff to add
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Init {} => {
            fs::create_dir_all(&*GIT_DIR.join("objects"))?;
            println!("Initialized empty Rustig repository in {:?}", *GIT_DIR);
        }

        Commands::HashObject { path } => {
            let file = File::open(path)?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;

            let oid: Digest = sha1_smol::Sha1::from(&contents).digest();
            let opath: &Path = &*GIT_DIR.join(oid.to_string());

            let object = File::create(opath)?;
            let mut object = BufWriter::new(object);
            object.write_all(&contents.as_bytes())?;
        }
    }

    Ok(())
}
