use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use assert_fs::prelude::*;
use predicates::prelude::predicate::{eq, path::exists, str::is_match};
use std::process::Command;

#[test]
fn subcommand_init_ok() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success()
        .stdout(is_match("^Initialized empty Rustig repository in /.*/\\.rustig\n$").unwrap());
    cwd.child(".rustig").assert(exists());
    cwd.child(".rustig/objects").assert(exists());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_hash_object_ok() -> Result<(), Box<dyn std::error::Error>> {
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
fn subcommand_hash_object_uninitialized_err() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("hash-object")
        .arg("nonexistent_file.txt")
        .assert()
        .code(eq(1))
        .stderr(is_match("^fatal: not a rustig repository\n$").unwrap());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_hash_object_nonexistent_file_err() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("hash-object")
        .arg("nonexistent_file.txt")
        .assert()
        .code(eq(1))
        .stderr(is_match("^fatal: could not read 'nonexistent_file.txt': No such file or directory \\(os error 2\\)\n$").unwrap());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_cat_file_ok() -> Result<(), Box<dyn std::error::Error>> {
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
fn subcommand_cat_file_uninitialized_err() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("cat-file")
        .arg("nonexistent_object")
        .assert()
        .code(eq(1))
        .stderr(is_match("^fatal: not a rustig repository\n$").unwrap());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_cat_file_nonexistent_object_err() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("cat-file")
        .arg("nonexistent_object")
        .assert()
        .code(eq(1))
        .stderr(is_match("^fatal: could not read object '/.*/nonexistent_object': No such file or directory \\(os error 2\\)\n$").unwrap());

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
        .stdout(is_match("^Initialized empty Rustig repository in /.*/\\.rustig\n$").unwrap());
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
