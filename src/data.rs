use lazy_static::lazy_static;
use sha1::{Digest, Sha1};
use std::{
    env,
    fs::{self},
    io::Result,
    path::PathBuf,
};

lazy_static! {
    pub static ref WORK_DIR: PathBuf = env::current_dir().unwrap(); // FIXME: set in main
    pub static ref REPO_DIR: PathBuf = WORK_DIR.join(".rustig");
    pub static ref OBJECTS_DIR: PathBuf = REPO_DIR.join("objects");
}

pub fn init() -> Result<String> {
    fs::create_dir_all(OBJECTS_DIR.as_path())?;
    Ok(REPO_DIR.display().to_string())
}

pub fn hash_object(path: PathBuf) -> Result<String> {
    let data = fs::read(path)?;

    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash = format!("{:x}", hasher.finalize());

    let path = OBJECTS_DIR.join(&hash);
    fs::write(path, data)?;

    Ok(hash)
}

pub fn cat_file(object: String) -> Result<String> {
    let path = OBJECTS_DIR.join(object);
    let data = fs::read_to_string(path)?;
    Ok(data)
}
