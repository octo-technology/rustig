use sha1::{Digest, Sha1};
use std::string::ToString;
use std::{
    fs::{self},
    io::Result,
    path::PathBuf,
};
use strum_macros::Display;

#[derive(Display, Debug)]
pub enum ObjectType {
    #[strum(serialize = "blob")]
    Blob,
}

pub struct Context {
    pub work_dir: PathBuf,
    pub repo_dir: PathBuf,
}

impl Context {
    pub fn init(&self) -> Result<String> {
        fs::create_dir_all(self.obj_dir().as_path())?;
        Ok(self.repo_dir.display().to_string())
    }

    pub fn hash_object(&self, path: PathBuf, type_: Option<ObjectType>) -> Result<String> {
        let object_data = fs::read_to_string(path)?;
        let object_type = type_.unwrap_or(ObjectType::Blob).to_string();
        let object = [object_type, object_data].join("\0");

        let mut hasher = Sha1::new();
        hasher.update(&object);
        let hash = format!("{:x}", hasher.finalize());

        let object_path = self.obj_dir().join(&hash);
        fs::write(object_path, object)?;

        Ok(hash)
    }

    pub fn get_object(&self, object: String) -> Result<String> {
        let path = self.obj_dir().join(object);
        let data = fs::read_to_string(path)?;
        Ok(data)
    }

    fn obj_dir(&self) -> PathBuf {
        self.repo_dir.join("objects")
    }
}
