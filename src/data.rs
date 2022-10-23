use anyhow::{anyhow, Context as Context_};
use async_recursion::async_recursion;
use sha1::{Digest, Sha1};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, SqliteConnection, Type};
use std::str::FromStr;
use std::string::ToString;
use std::{fs, path::PathBuf};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Type)]
#[sqlx(transparent)]
pub struct OID(pub String);

impl std::fmt::Display for OID {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Display, Debug, PartialEq, EnumString, Eq, PartialOrd, Ord, Type)]
#[sqlx(rename_all = "lowercase")]
pub enum ObjectType {
    #[strum(serialize = "blob")]
    Blob,

    #[strum(serialize = "tree")]
    Tree,
}

pub struct Context {
    ignored: Vec<PathBuf>,
    conn: SqliteConnection,
}

impl Context {
    // FIXME: error messages don't maeke sense.
    pub async fn new(repo_file: PathBuf, init: bool) -> anyhow::Result<Self> {
        log::trace!("Building execution context");
        let mut conn = SqliteConnectOptions::from_str(&repo_file.to_string_lossy())
            .context("not a valid repository path")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Delete)
            .create_if_missing(init)
            .connect()
            .await
            .map_err(|e| {
                log::debug!("{e}");
                if init {
                    anyhow!("cannot initialize rustig repository")
                } else {
                    anyhow!("not a rustig repository")
                }
            })?;

        if init {
            sqlx::migrate!()
                .run_direct(&mut conn)
                .await
                .context("could not run migrations")?;
        }

        Ok(Self {
            conn,
            ignored: vec![repo_file
                .canonicalize()
                .context("not a valid repository path")?],
        })
    }

    pub async fn hash_object(&mut self, data: Vec<u8>, typ: ObjectType) -> anyhow::Result<OID> {
        let mut hasher = Sha1::new();
        hasher.update(typ.to_string().as_bytes());
        hasher.update("\0"); // TODO: this is here to not break tests, but we should remove it
        hasher.update(&data);
        let hash = OID(format!("{:x}", hasher.finalize()));

        sqlx::query!(
            "INSERT INTO objects(id, type, data) VALUES (?1, ?2, ?3)",
            hash,
            typ,
            data
        )
        .execute(&mut self.conn)
        .await
        .context(format!("could not write object '{}'", hash))?;

        Ok(hash)
    }

    pub async fn get_object(
        &mut self,
        object: &OID,
        expected: &[ObjectType],
    ) -> anyhow::Result<Vec<u8>> {
        let record = sqlx::query!(
            "SELECT type AS typ, data FROM objects WHERE id = ?1",
            object
        )
        .fetch_one(&mut self.conn)
        .await
        .context(format!("could not read object '{}'", object))?;

        let object_type = ObjectType::from_str(&record.typ).map_err(|_| {
            anyhow!(
                "could not parse object '{}': unknown type '{}'",
                object,
                record.typ
            )
        })?;

        if !expected.is_empty() && !expected.contains(&object_type) {
            Err(anyhow!(
                "invalid object type for '{}': want one of [{}] but got '{}'",
                object,
                expected
                    .into_iter()
                    .map(|e| format!("'{}'", e))
                    .collect::<Vec<String>>()
                    .join(", "),
                object_type
            ))
        } else {
            Ok(record.data)
        }
    }

    #[async_recursion]
    pub async fn write_tree(&mut self, path: &PathBuf) -> anyhow::Result<OID> {
        let path = path.canonicalize().context("not a valid path")?;

        let mut entries = vec![];
        let files = fs::read_dir(&path)
            .context(format!("could not read '{}'", path.display()))?
            .filter_map(|e| e.ok())
            .filter(|f| !self.is_ignored(&f.path()))
            .collect::<Vec<_>>();

        for f in files {
            if f.file_type().map_or(false, |t| t.is_dir()) {
                let oid = self.write_tree(&f.path()).await?;
                entries.push((ObjectType::Tree, oid, f.file_name()));
            } else {
                let data = fs::read(f.path())
                    .context(format!("could not read file '{}'", f.path().display()))?;
                let oid = self.hash_object(data, ObjectType::Blob).await?;
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
        self.hash_object(data, ObjectType::Tree).await
    }

    #[async_recursion]
    pub async fn read_tree(&mut self, object: OID, path: &PathBuf) -> anyhow::Result<()> {
        let data_raw = &self.get_object(&object, &[ObjectType::Tree]).await?;
        let data = String::from_utf8_lossy(data_raw);

        if data == "" {
            // tree objects containing no entries (i.e. empty dirs)
            return Ok(());
        }
        for e in data.split("\n") {
            let (type_str, rest) = e.split_once('\0').context(format!(
                "could not parse object '{}': invalid format",
                object
            ))?;
            let (oid, name) = {
                let (o, n) = rest.split_once('\0').context(format!(
                    "could not parse object '{}': invalid format",
                    object
                ))?;
                (OID(o.to_string()), n)
            };
            let object_type = ObjectType::from_str(type_str).map_err(|_| {
                anyhow!(
                    "could not parse object '{}': unknown type '{}'",
                    object,
                    type_str
                )
            })?;

            let new_path = path.join(name);
            match object_type {
                ObjectType::Blob => {
                    let new_data = self.get_object(&oid, &[ObjectType::Blob]).await?;
                    fs::write(&new_path, new_data)
                        .context(format!("could not write file '{}'", new_path.display()))?;
                }
                ObjectType::Tree => {
                    fs::create_dir(&new_path).context(format!(
                        "could not create directory '{}'",
                        new_path.display()
                    ))?;
                    self.read_tree(oid, &new_path).await?;
                }
            };
        }

        Ok(())
    }

    // Paths need to be canonical: ./foo does not starts with foo.
    fn is_ignored(&self, path: &PathBuf) -> bool {
        self.ignored.iter().any(|f| path.starts_with(f))
    }
}
