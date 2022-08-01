use anyhow::{anyhow, Context as Context_};
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::string::ToString;
use std::{fs, path::PathBuf};
use strum_macros::{Display, EnumString};

#[derive(Display, Debug, PartialEq, EnumString)]
pub enum ObjectType {
    #[strum(serialize = "blob")]
    Blob,

    #[strum(serialize = "tree")]
    Tree,
}

pub struct Context {
    pub work_dir: PathBuf,
    pub repo_dir: PathBuf,
}

impl Context {
    pub fn ensure_init(&self) -> anyhow::Result<()> {
        self.obj_dir()
            .as_path()
            .is_dir()
            .then(|| ())
            .ok_or(anyhow::Error::msg("not a rustig repository"))
    }

    pub fn init(&self) -> anyhow::Result<String> {
        fs::create_dir_all(self.obj_dir().as_path()).context(format!(
            "could not create directory '{}'",
            self.obj_dir().display()
        ))?;
        Ok(self.repo_dir.display().to_string())
    }

    pub fn hash_object(&self, data: String, type_: ObjectType) -> anyhow::Result<String> {
        let object = [type_.to_string(), data].join("\0");

        let mut hasher = Sha1::new();
        hasher.update(&object);
        let hash = format!("{:x}", hasher.finalize());

        let path = self.obj_dir().join(&hash);
        fs::write(&path, object).context(format!("could not write object '{}'", path.display()))?;

        Ok(hash)
    }

    pub fn get_object(
        &self,
        object: String,
        expected: Option<ObjectType>,
    ) -> anyhow::Result<String> {
        let object_path = self.obj_dir().join(object);
        let object_content = fs::read_to_string(&object_path)
            .context(format!("could not read object '{}'", object_path.display()))?;
        let (object_type_str, object_data) = object_content.split_once('\0').context(format!(
            "could not parse object '{}': invalid format",
            object_path.display()
        ))?;
        let object_type = ObjectType::from_str(object_type_str).map_err(|_| {
            anyhow!(
                "could not parse object '{}': unknown type '{}'",
                object_path.display(),
                object_type_str
            )
        })?;

        match expected {
            Some(e) if e != object_type => Err(anyhow!(
                "could not parse object '{}': expected type '{}' but got '{}'",
                object_path.display(),
                object_type_str,
                e.to_string()
            )),
            _ => Ok(object_data.to_string()),
        }
    }

    pub fn write_tree(&self, path: &PathBuf) -> anyhow::Result<String> {
        let mut entries = vec![];
        let files = fs::read_dir(path)
            .context(format!("could not read '{}'", path.display()))?
            .filter_map(|e| e.ok())
            .filter(|f| !self.is_ignored(&f.path()));

        for f in files {
            if f.file_type().map_or(false, |t| t.is_dir()) {
                let oid = self.write_tree(&f.path())?;
                entries.push((ObjectType::Tree, oid, f.file_name()));
            } else {
                let data = fs::read_to_string(f.path())
                    .context(format!("could not read '{}'", f.path().display()))?;
                let oid = self.hash_object(data, ObjectType::Blob)?;
                entries.push((ObjectType::Blob, oid, f.file_name()));
            }
        }

        let mut data = entries
            .into_iter()
            .map(|e| format!("{}\0{}\0{}", e.0, e.1, e.2.to_string_lossy()))
            .collect::<Vec<String>>();
        data.sort();
        self.hash_object(data.join("\n"), ObjectType::Tree)
    }

    fn is_ignored(&self, path: &PathBuf) -> bool {
        path.starts_with(&self.repo_dir)
    }

    fn obj_dir(&self) -> PathBuf {
        self.repo_dir.join("objects")
    }
}
