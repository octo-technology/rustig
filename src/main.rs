mod cli;
mod data;
mod error;

fn main() {
    std::process::exit(match cli::parse() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{:?}", err);
            -1
        }
    });
}
