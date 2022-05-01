use data_encoding::HEXUPPER;
use sha1::{Digest, Sha1};
use std::{fs, io, path};

pub const GIT_DIR: &str = ".rustig";

pub fn init() -> io::Result<()> {
    fs::create_dir(GIT_DIR)?;
    let obj_path = path::PathBuf::from(format!("{GIT_DIR}/objects"));
    fs::create_dir(obj_path)?;
    Ok(())
}

pub fn hash_object(path: path::PathBuf) -> io::Result<String> {
    let content = fs::read_to_string(path).expect("Unable to read file");

    let hash = Sha1::new().chain_update(&content).finalize();
    let oid = HEXUPPER.encode(hash.as_ref());

    let output_path = path::PathBuf::from(format!("{GIT_DIR}/objects/{oid}"));
    fs::write(output_path, content).expect("Unable to write file");

    Ok(oid)
}
