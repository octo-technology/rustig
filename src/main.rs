mod cli;
mod data;

fn main() {
    cli::parse().expect("Unable to execute the command")
}
