mod cli;
mod data;

fn main() {
    std::process::exit(match cli::parse() {
        Ok(_) => 0,
        Err(err) => {
            println!("fatal: {}", err);
            -1
        }
    });
}
