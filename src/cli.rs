use crate::data;
use std::{env, io};

pub fn parse() -> io::Result<()> {
    for argument in std::env::args().skip(1) {
        match argument {
            arg if arg == "init" => {
                data::init()?;
                let current_directory = env::current_dir()?;
                println!(
                    "Initialized empty rustig repository in {}/{}",
                    current_directory.display(),
                    data::GIT_DIR
                )
            }
            _ => default(argument),
        }
    }
    Ok(())
}

fn default(arg: String) {
    println!("Unknown command: {}", arg);
}
