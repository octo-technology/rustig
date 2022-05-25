use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use assert_fs::prelude::*;
use predicates::prelude::predicate::{path::exists, str::is_match};
use std::process::Command;

#[test]
fn subcommand_init() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success()
        .stdout(is_match("^Initialized empty Rustig repository in .*/\\.rustig\n$").unwrap());
    cwd.child(".rustig").assert(exists());
    cwd.child(".rustig/objects").assert(exists());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_hash_object() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let file = cwd.child("some_file.txt");
    file.write_str("Some content.\n")?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("hash-object")
        .arg(file.path())
        .assert()
        .success()
        .stdout(is_match("^d727e363541ff1b8b282bde54a780d05e8007a8f\n$").unwrap());
    cwd.child(".rustig/objects/d727e363541ff1b8b282bde54a780d05e8007a8f")
        .assert(exists());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_cat() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let file = cwd.child("some_file.txt");
    file.write_str("Some content.\n")?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("hash-object")
        .arg(file.path())
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("cat-file")
        .arg("d727e363541ff1b8b282bde54a780d05e8007a8f")
        .assert()
        .success()
        .stdout(is_match("^Some content.\n\n$").unwrap());

    cwd.close()?;
    Ok(())
}

#[test]
fn simple_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let file = cwd.child("some_file.txt");
    file.write_str("Some content.\n")?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success()
        .stdout(is_match("^Initialized empty Rustig repository in .*/\\.rustig\n$").unwrap());
    cwd.child(".rustig").assert(exists());
    cwd.child(".rustig/objects").assert(exists());

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("hash-object")
        .arg(file.path())
        .assert()
        .success()
        .stdout(is_match("^d727e363541ff1b8b282bde54a780d05e8007a8f\n$").unwrap());
    cwd.child(".rustig/objects/d727e363541ff1b8b282bde54a780d05e8007a8f")
        .assert(exists());

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("cat-file")
        .arg("d727e363541ff1b8b282bde54a780d05e8007a8f")
        .assert()
        .success()
        .stdout(is_match("^Some content.\n\n$").unwrap());

    cwd.close()?;
    Ok(())
}

#[test]
fn cli_tests() {
    trycmd::TestCases::new().case("tests/cmd/*.trycmd");
}
