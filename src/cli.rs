use crate::data::{self, ObjectType, OID};
use anyhow::{Context, Ok};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use prost::Message;
use std::io::{Cursor, Read, Write};
use std::{env, fs, path::PathBuf};
use wasmer::{Instance, Module, Store};
use wasmer_wasi::{WasiBidirectionalSharedPipePair, WasiState};

pub mod hook {
    include!(concat!(env!("OUT_DIR"), "/hook.rs"));
}

#[derive(Parser, Debug)]
#[command(about = "A bad git clone, in Rust", long_about = None)]
struct Cli {
    /// The command to run
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an empty rustig repository
    Init,

    /// Compute object ID and create a blob from a file
    #[command(name = "hash-object", arg_required_else_help = true)]
    HashObject {
        /// File to hash
        #[arg(required = true)]
        path: PathBuf,
    },

    /// Provide content for repository objects
    #[command(name = "cat-file", arg_required_else_help = true)]
    CatFile {
        /// Object to show
        #[arg(required = true)]
        object: String,
    },

    /// Create a tree object from the current index
    #[command(name = "write-tree")]
    WriteTree,

    /// Read tree information into the index
    #[command(name = "read-tree", arg_required_else_help = true)]
    ReadTree {
        /// Object to read
        #[arg(required = true)]
        object: String,
    },
}

pub fn parse() -> anyhow::Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    log::trace!("Building execution context");
    let work_dir = env::current_dir().context("could not resolve current working directory")?;
    let context = data::Context {
        work_dir: PathBuf::from(&work_dir), // TODO: supplyable via global flag `--work-tree`
        repo_dir: work_dir.join(".rustig"), // TODO: supplyable via global flag `--git-dir`
    };

    return match args.command {
        Commands::Init => init(&context),
        Commands::HashObject { path } => hash_object(&context, path),
        Commands::CatFile { object } => cat_file(&context, object),
        Commands::WriteTree => write_tree(&context),
        Commands::ReadTree { object } => read_tree(&context, object),
    };
}

fn init(context: &data::Context) -> anyhow::Result<()> {
    println!("Initialized empty Rustig repository in {}", context.init()?);
    Ok(())
}

fn hash_object(context: &data::Context, path: PathBuf) -> anyhow::Result<()> {
    context.ensure_init()?;
    let data = fs::read(&path).context(format!("could not read '{}'", path.display()))?;
    println!("{}", context.hash_object(data, ObjectType::Blob)?);
    hook()?;
    Ok(())
}

fn cat_file(context: &data::Context, object: String) -> anyhow::Result<()> {
    context.ensure_init()?;
    let data = context.get_object(OID(object), &[ObjectType::Blob])?;
    println!("{}", String::from_utf8_lossy(&data));
    hook()?;
    Ok(())
}

fn write_tree(context: &data::Context) -> anyhow::Result<()> {
    context.ensure_init()?;
    println!("{}", context.write_tree(&context.work_dir)?);
    hook()?;
    Ok(())
}

fn read_tree(context: &data::Context, object: String) -> anyhow::Result<()> {
    context.ensure_init()?;
    context.read_tree(OID(object), &context.work_dir)?;
    hook()?;
    Ok(())
}

fn hook() -> anyhow::Result<()> {
    let mut store = Store::default();
    let module = Module::from_file(&store, "target/wasm32-wasi/debug/hook.wasm")?;
    let mut input = WasiBidirectionalSharedPipePair::new().with_blocking(false);
    let mut output = WasiBidirectionalSharedPipePair::new().with_blocking(false);
    let wasi_env = WasiState::new("hello")
        .stdin(Box::new(input.clone()))
        .stdout(Box::new(output.clone()))
        .finalize(&mut store)?;

    let import_object = wasi_env.import_object(&mut store, &module)?;
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let memory = instance.exports.get_memory("memory")?;
    wasi_env.data_mut(&mut store).set_memory(memory.clone());

    let mut req = hook::Req::default();
    req.foo = String::from("Hello world");

    let mut req_b = Vec::new();
    req_b.reserve(req.encoded_len());
    req.encode(&mut req_b).unwrap();
    input.write_all(&req_b)?;

    instance
        .exports
        .get_function("_start")?
        .call(&mut store, &[])?;

    let mut resp_b = Vec::new();
    output.read_to_end(&mut resp_b)?;

    let resp = hook::Resp::decode(&mut Cursor::new(resp_b)).unwrap();

    println!("Read \"{:?}\" from the WASI stdout!", resp);

    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
