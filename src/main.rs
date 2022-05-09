mod cli;
mod data;

fn main() {
    cli::parse().unwrap_or_else(|err| println!("{:?}", err))
}
