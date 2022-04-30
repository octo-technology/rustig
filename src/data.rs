use sha1::{Sha1, Digest};
use std::{fs, path, error};
use base16ct;

const RUSTIG_DIR: &str = ".rustig";
const OBJECTS_DIR: &str = "objects";

pub fn init() -> Result<(), Box<dyn error::Error>> {
    fs::create_dir(RUSTIG_DIR)?;
    fs::create_dir(path::PathBuf::from(format!("{RUSTIG_DIR}/objects")))?;
    Ok(())
}

pub fn hash_object(path: &path::PathBuf) -> Result<(), Box<dyn error::Error>> {
    // let mut hasher = Sha1::new();
    // let mut file = fs::File::open(path)?;
    // io::copy(&mut file, &mut hasher)?;
    // let oid = base16ct::lower::encode_string(&hasher.finalize());
    // fs::copy(path, format!("{RUSTIG_DIR}/objects/{oid}"))?;
    // print!("{oid}");

    let content = fs::read_to_string(path)?;
    let hash = Sha1::new().chain_update(&content).finalize();
    let oid = base16ct::lower::encode_string(&hash);
    let output_path = path::PathBuf::from(format!("{RUSTIG_DIR}/{OBJECTS_DIR}/{oid}"));
    fs::write(output_path, content)?;
    println!("OID: {oid}");
    Ok(())
}

pub fn cat_object(oid: &String) -> Result<(), Box<dyn error::Error>> {
    let output_path = path::PathBuf::from(format!("{RUSTIG_DIR}/{OBJECTS_DIR}/{oid}"));
    let content = fs::read_to_string(output_path)?;
    print!("{content}");
    Ok(())
}
