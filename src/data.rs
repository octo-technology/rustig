use anyhow::{anyhow, Context as Context_};
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::string::ToString;
use std::{fs, path::PathBuf};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OID(pub String);

impl std::fmt::Display for OID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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

    pub fn hash_object(&self, data: Vec<u8>, typ: ObjectType) -> anyhow::Result<OID> {
        let object = [typ.to_string().as_bytes(), &[b'\0'], &data].concat();

        let mut hasher = Sha1::new();
        hasher.update(&object);
        let hash = format!("{:x}", hasher.finalize());

        let path = self.obj_dir().join(&hash);
        fs::write(&path, object).context(format!("could not write object '{}'", path.display()))?;

        Ok(OID(hash))
    }

    pub fn get_object(&self, object: OID, expected: &[ObjectType]) -> anyhow::Result<Vec<u8>> {
        let object_path = self.obj_dir().join(object.0);
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

        if !expected.is_empty() && !expected.contains(&object_type) {
            Err(anyhow!(
                "could not parse object '{}': expected type to be one of [{}] but got '{}'",
                object_path.display(),
                expected
                    .into_iter()
                    .map(|e| format!("'{}'", e))
                    .collect::<Vec<String>>()
                    .join(", "),
                object_type_str
            ))
        } else {
            Ok(object_data[1..].to_vec())
        }
    }

    pub fn write_tree(&self, path: &PathBuf) -> anyhow::Result<OID> {
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

    pub fn read_tree(&self, object: OID, path: &PathBuf) -> anyhow::Result<()> {
        let object_path = self.obj_dir().join(&object.0);
        let data_raw = self.get_object(object, &[ObjectType::Tree])?;
        let data = String::from_utf8_lossy(&data_raw);

        if data == "" {
            // tree objects containing no entries (i.e. empty dirs)
            return Ok(());
        }
        for e in data.split("\n") {
            let (type_str, rest) = e.split_once('\0').context(format!(
                "could not parse object '{}': invalid format",
                object_path.display()
            ))?;
            let (oid, name) = {
                let (o, n) = rest.split_once('\0').context(format!(
                    "could not parse object '{}': invalid format",
                    object_path.display()
                ))?;
                (OID(o.to_string()), n)
            };
            let object_type = ObjectType::from_str(type_str).map_err(|_| {
                anyhow!(
                    "could not parse object '{}': unknown type '{}'",
                    object_path.display(),
                    type_str
                )
            })?;

            let new_path = path.join(name);
            match object_type {
                ObjectType::Blob => {
                    let new_data = self.get_object(oid, &[ObjectType::Blob])?;
                    fs::write(&new_path, new_data)
                        .context(format!("could not write file '{}'", new_path.display()))?;
                }
                ObjectType::Tree => {
                    fs::create_dir(&new_path).context(format!(
                        "could not create directory '{}'",
                        new_path.display()
                    ))?;
                    self.read_tree(oid, &new_path)?;
                }
            };
        }

        Ok(())
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
