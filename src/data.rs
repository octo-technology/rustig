use sha1::{Digest, Sha1};
use std::{
    fs::{self},
    io::Result,
    path::PathBuf,
};

pub struct Context {
    pub work_dir: PathBuf,
    pub repo_dir: PathBuf,
}

impl Context {
    pub fn init(&self) -> Result<String> {
        fs::create_dir_all(self.obj_dir().as_path())?;
        Ok(self.repo_dir.display().to_string())
    }

    pub fn hash_object(&self, path: PathBuf) -> Result<String> {
        let data = fs::read(path)?;

        let mut hasher = Sha1::new();
        hasher.update(&data);
        let hash = format!("{:x}", hasher.finalize());

        let path = self.obj_dir().join(&hash);
        fs::write(path, data)?;

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
