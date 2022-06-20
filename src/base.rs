use std::{env, fs, path::PathBuf};

use anyhow;

use crate::data::GIT_DIR;

pub fn write_tree(directory: Option<String>) -> anyhow::Result<()> {
    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
    let dir = directory.unwrap_or(current_dir);

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;

        if is_ignored(&path) {
            continue;
        }

        if metadata.is_file() {
            println!("{}", path.display());
        } else if metadata.is_dir() {
            let path_as_string = path.as_path().to_str().unwrap().to_string();
            write_tree(Some(path_as_string))?;
        }
    }

    Ok(())
}

fn is_ignored(path: &PathBuf) -> bool {
    return path.iter().any(|x| x == GIT_DIR.file_name().unwrap());
}
