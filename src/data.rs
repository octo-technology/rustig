use lazy_static::lazy_static;
use sha1_smol::Digest;
use std::{
    env,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

lazy_static! {
    pub static ref GIT_DIR: PathBuf = env::current_dir().unwrap().join(".rustig");
    pub static ref GIT_OBJECTS_DIR: PathBuf = GIT_DIR.join("objects");
}

pub fn init() -> io::Result<String> {
    fs::create_dir_all(GIT_OBJECTS_DIR.as_path())?;
    Ok(GIT_DIR.display().to_string())
}

pub fn hash_object(path: PathBuf) -> io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let oid: Digest = sha1_smol::Sha1::from(&contents).digest();
    let opath: PathBuf = GIT_OBJECTS_DIR.join(oid.to_string());

    let object = File::create(opath)?;

    let mut object = BufWriter::new(object);
    object.write_all(contents.as_bytes())?;

    Ok(oid.to_string())
}

pub fn cat_file(object: String) -> io::Result<String> {
    let file = File::open(GIT_OBJECTS_DIR.join(object))?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}
