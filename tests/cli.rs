use assert_cmd::prelude::*; // Add methods on commands
use assert_fs::{prelude::*, TempDir};
use predicates::prelude::*; // Used for writing assertions
use std::{env, process::Command}; // Run programs

fn create_and_set_current_dir(create_objects: bool) -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    env::set_current_dir(&temp)?;

    if create_objects {
        temp.child(".rustig/objects").create_dir_all()?;
    }

    return Ok(temp);
}

#[test]
fn init_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    // given
    let mut cmd = Command::cargo_bin("rustig")?;
    let temp = create_and_set_current_dir(false)?;

    // when
    cmd.arg("init");

    // then
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));
    temp.child(".rustig").assert(predicate::path::exists());
    temp.child(".rustig/objects")
        .assert(predicate::path::exists());
    temp.close()?;

    Ok(())
}

#[test]
fn hash_object_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    // given
    let mut cmd = Command::cargo_bin("rustig")?;
    let temp = create_and_set_current_dir(true)?;
    let temp_file = temp.child("test.txt");
    temp_file.write_str("Bacon ipsum dolor amet doner pork chop filet mignon beef ribs.\n")?;

    // when
    cmd.arg("hash-object").arg(temp_file.path());

    // then
    cmd.assert().success().stdout(predicate::str::contains(
        "cc67029eb5860e56e3ccefaf6036e80380fe8372",
    ));
    temp.child(".rustig/objects/cc67029eb5860e56e3ccefaf6036e80380fe8372")
        .assert(predicate::path::exists());
    temp.close()?;

    Ok(())
}

#[test]
fn cat_file_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    // given
    let mut cmd = Command::cargo_bin("rustig")?;
    let temp = create_and_set_current_dir(false)?;
    let temp_file = temp.child(".rustig/objects/cc67029eb5860e56e3ccefaf6036e80380fe8372");
    temp_file.write_str("Bacon ipsum dolor amet doner pork chop filet mignon beef ribs.\n")?;

    // when
    cmd.arg("cat-file").arg(temp_file.path());

    // then
    cmd.assert().success().stdout(predicate::str::contains(
        "Bacon ipsum dolor amet doner pork chop filet mignon beef ribs.\n",
    ));
    temp.close()?;

    Ok(())
}
