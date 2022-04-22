use std::io;

pub fn init() -> io::Result<()> {
    // AlreadyExists error not dealt with
    std::fs::create_dir(".rustig").unwrap();
    Ok(())
}
