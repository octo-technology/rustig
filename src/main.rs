use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use sha1_smol::Digest;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{env, fs};

lazy_static! {
    pub static ref GIT_DIR: PathBuf = env::current_dir().unwrap().join(".rustig");
    pub static ref GIT_OBJECTS_DIR: PathBuf = GIT_DIR.join("objects");
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
        /// Object to hash
        #[clap(required = true, parse(from_os_str))]
        path: PathBuf,
    },

    /// Print object to stdout
    #[clap(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        /// Object to print
        #[clap(required = true)]
        object: String,
    },
}

fn write_to_stdout(bytes: &[u8]) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.flush()?;
    handle.write_all(&bytes)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Init {} => {
            fs::create_dir_all(&*GIT_OBJECTS_DIR)?;
            println!("Initialized empty Rustig repository in {:?}", *GIT_DIR);
        }

        Commands::HashObject { path } => {
            let file = File::open(path)?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;

            let oid: Digest = sha1_smol::Sha1::from(&contents).digest();
            let opath: PathBuf = GIT_OBJECTS_DIR.join(oid.to_string());

            let object = File::create(opath)?;

            let mut object = BufWriter::new(object);
            object.write_all(contents.as_bytes())?;

            write_to_stdout(&oid.to_string().as_bytes())?;
        }

        Commands::CatFile { object } => {
            let file = File::open(GIT_OBJECTS_DIR.join(object))?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;

            write_to_stdout(contents.as_bytes())?;
        }
    }

    Ok(())
}
