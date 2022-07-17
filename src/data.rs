use anyhow::{anyhow, Context as Context_};
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::string::ToString;
use std::{fs, path::PathBuf};
use strum_macros::{Display, EnumString};

#[derive(Display, Debug, PartialEq, EnumString, Eq, PartialOrd, Ord)]
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

    pub fn hash_object(&self, data: Vec<u8>, typ: ObjectType) -> anyhow::Result<String> {
        let object = [typ.to_string().as_bytes(), &[b'\0'], &data].concat();

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
    ) -> anyhow::Result<Vec<u8>> {
        let object_path = self.obj_dir().join(object);
        let object_content = fs::read(&object_path)
            .context(format!("could not read object '{}'", object_path.display()))?;

        let position = object_content
            .iter()
            .position(|&e| e == b'\0')
            .context(format!(
                "could not parse object '{}': invalid format",
                object_path.display()
            ))?;
        let (object_type_raw, object_data) = object_content.split_at(position);

        let object_type_str = std::str::from_utf8(object_type_raw).context(format!(
            "could not parse object '{}': invalid object type",
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
                e.to_string(),
                object_type_str
            )),
            _ => Ok(object_data[1..].to_vec()),
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
                let data = fs::read(f.path())
                    .context(format!("could not read file '{}'", f.path().display()))?;

                let oid = self.hash_object(data, ObjectType::Blob)?;
                entries.push((ObjectType::Blob, oid, f.file_name()));
            }
        }

        entries.sort();
        let data = entries
            .into_iter()
            .map(|e| format!("{}\0{}\0{}", e.0, e.1, e.2.to_string_lossy()))
            .collect::<Vec<String>>()
            .join("\n")
            .into_bytes();
        self.hash_object(data, ObjectType::Tree)
    }

    fn is_ignored(&self, path: &PathBuf) -> bool {
        path.starts_with(&self.repo_dir)
            || path.starts_with(&self.work_dir.join("target"))
            || path.starts_with(&self.work_dir.join(".git"))
    }

    fn obj_dir(&self) -> PathBuf {
        self.repo_dir.join("objects")
    }
}
