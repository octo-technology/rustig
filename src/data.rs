use anyhow;
use lazy_static::lazy_static;
use sha1_smol;
use std::{
    env,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use crate::error::CliError;

lazy_static! {
    pub static ref GIT_DIR: PathBuf = env::current_dir().unwrap().join(".rustig");
    pub static ref GIT_OBJECTS_DIR: PathBuf = GIT_DIR.join("objects");
}

pub fn init() -> anyhow::Result<String> {
    fs::create_dir_all(GIT_OBJECTS_DIR.as_path())?;
    Ok(GIT_DIR.display().to_string())
}

pub fn hash_object(path: PathBuf, type_: Option<String>) -> anyhow::Result<String> {
    let object_type = type_.unwrap_or("blob".to_string());
    let file: File = File::open(path)?;
    let mut buf_reader: BufReader<File> = BufReader::new(file);
    let mut file_content: String = String::new();
    buf_reader.read_to_string(&mut file_content)?;

    let contents: String = format!("{object_type}\0{file_content}");

    let oid: String = sha1_smol::Sha1::from(&contents.as_bytes())
        .digest()
        .to_string();
    let opath: PathBuf = GIT_OBJECTS_DIR.join(&oid);
    let object: File = File::create(opath)?;
    let mut object: BufWriter<File> = BufWriter::new(object);
    object.write_all(contents.as_bytes())?;

    Ok(oid)
}

pub fn cat_file(object: String, expected: Option<String>) -> anyhow::Result<String> {
    let expected_type = expected.unwrap_or("blob".to_string());
    let file: File = File::open(GIT_OBJECTS_DIR.join(object))?;
    let mut buf_reader: BufReader<File> = BufReader::new(file);
    let mut file_content: String = String::new();
    buf_reader.read_to_string(&mut file_content)?;

    let contents_splited: Vec<&str> = file_content.split("\0").collect();
    let type_: String = String::from(contents_splited[0]);
    let data: String = String::from(contents_splited[1]);

    if expected_type != type_ {
        return Err(anyhow::anyhow!(CliError::InvalidType {
            expected: expected_type,
            found: type_
        }));
    }

    Ok(data)
}
