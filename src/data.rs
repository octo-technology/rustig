use std::io;

pub const GIT_DIR: &str = ".rustig";

pub fn init() -> io::Result<()> {
    std::fs::create_dir(GIT_DIR)?;
    Ok(())
}
