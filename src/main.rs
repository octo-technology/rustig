mod data;

use clap::Parser;
use std::io;

// Implem
#[derive(Parser, Debug)]
struct Cli {
    command: String,
}

fn default(wrong_command: String) -> io::Result<()> {
    println!("No command called : '{}'", wrong_command);
    Ok(())
}

fn main() {
    let args = Cli::parse();
    match args.command.as_str() {
        "init" => data::init(),
        _ => default(args.command),
    };
}
