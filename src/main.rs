mod cli;
mod data;

#[tokio::main]
async fn main() {
    std::process::exit(match cli::parse().await {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("fatal: {:#}", err);
            1
        }
    });
}
