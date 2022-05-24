use anyhow::Context as Context_;
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::string::ToString;
use std::{
    fs::{self},
    path::PathBuf,
};
use strum_macros::{Display, EnumString};

#[derive(Display, Debug, PartialEq, EnumString)]
pub enum ObjectType {
    #[strum(serialize = "blob")]
    Blob,
}

pub struct Context {
    pub work_dir: PathBuf,
    pub repo_dir: PathBuf,
}

impl Context {
    pub fn init(&self) -> anyhow::Result<String> {
        fs::create_dir_all(self.obj_dir().as_path()).context(format!(
            "could not create directory '{}'",
            self.obj_dir().display()
        ))?;
        Ok(self.repo_dir.display().to_string())
    }

    pub fn hash_object(&self, path: PathBuf, type_: Option<ObjectType>) -> anyhow::Result<String> {
        let object_data =
            fs::read_to_string(&path).context(format!("could not read '{}'", path.display()))?;
        let object_type = type_.unwrap_or(ObjectType::Blob).to_string();
        let object = [object_type, object_data].join("\0");

        let mut hasher = Sha1::new();
        hasher.update(&object);
        let hash = format!("{:x}", hasher.finalize());

        let object_path = self.obj_dir().join(&hash);
        fs::write(&object_path, object).context(format!(
            "could not write object '{}'",
            object_path.display()
        ))?;

        Ok(hash)
    }

    pub fn get_object(
        &self,
        object: String,
        expected: Option<ObjectType>,
    ) -> anyhow::Result<String> {
        let object_path = self.obj_dir().join(object);
        let object = fs::read_to_string(&object_path)
            .context(format!("could not read object '{}'", object_path.display()))?;
        let (object_type, object_data) = object.split_once('\0').context(format!(
            "could not parse object '{}'",
            object_path.display()
        ))?;
        // FIXME: `from_str` returns unclear error msg: "Matching variant not found".
        //  Replace it with "unrecognized type '{}'".
        let object_type_enum = ObjectType::from_str(object_type).context(format!(
            "could not parse object '{}'",
            object_path.display()
        ))?;

        if let Some(x) = expected {
            if x != object_type_enum {
                return Err(anyhow::Error::msg(format!(
                    "could not parse object '{}': expected type '{}' but got '{}'",
                    object_path.display(),
                    object_type,
                    x.to_string()
                )));
            }
        }

        Ok(object_data.to_string())
    }

    fn obj_dir(&self) -> PathBuf {
        self.repo_dir.join("objects")
    }
}
