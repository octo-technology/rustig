use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use assert_fs::prelude::*;
use predicates::{
    prelude::predicate::{eq, path::exists, str::is_match},
    str::is_empty,
};
use std::process::Command;

#[test]
fn missing_subcommand_err() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let help_msg = "\
rustig \
\nA bad git clone, in Rust

USAGE:
    rustig [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -q, --quiet      Less output per occurrence
    -v, --verbose    More output per occurrence

SUBCOMMANDS:
    cat-file       Provide content for repository objects
    hash-object    Compute object ID and create a blob from a file
    help           Print this message or the help of the given subcommand(s)
    init           Create an empty rustig repository
    read-tree      Read tree information into the index
    write-tree     Create a tree object from the current index
";

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .assert()
        .code(2)
        .stderr(eq(help_msg));

    cwd.close()?;
    Ok(())
}

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
fn subcommand_write_tree_ok() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let d1 = cwd.child("some_dir");
    let d2 = d1.child("some_nested_dir");
    let f1 = cwd.child("some_file.txt");
    let f2 = d1.child("some_nested_file.txt");
    let f3 = d1.child("another_nested_file.txt");
    d1.create_dir_all()?;
    d2.create_dir_all()?;
    f1.write_str("Some content.\n")?;
    f2.write_str("Some more content.\n")?;
    f3.write_str("Yet more content.\n")?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("write-tree")
        .assert()
        .success()
        .stdout(is_match("^dcb3fb5c65318b85054a7a687555cbabaf103969\n$").unwrap());
    cwd.child(".rustig/objects/523b2350136645d418734c8f8eef0be1e7e09861")
        .assert(exists());
    cwd.child(".rustig/objects/2276357f8ac1bc0b174c9ccbea7fcdbeaf2be70b")
        .assert(exists());
    cwd.child(".rustig/objects/a5b3c94541feba5ee9b0749d2f8ba380ec5b07f2")
        .assert(exists());
    cwd.child(".rustig/objects/d28c5ff92df044a522508a29cf3fad0b812f672f")
        .assert(exists());
    cwd.child(".rustig/objects/d727e363541ff1b8b282bde54a780d05e8007a8f")
        .assert(exists());
    cwd.child(".rustig/objects/dcb3fb5c65318b85054a7a687555cbabaf103969")
        .assert(exists());

    cwd.close()?;
    Ok(())
}

#[test]
fn subcommand_read_tree_ok() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = assert_fs::TempDir::new()?;
    let d1 = cwd.child("some_dir");
    let d2 = d1.child("some_nested_dir");
    let f1 = cwd.child("some_file.txt");
    let f2 = d1.child("some_nested_file.txt");
    let f3 = d1.child("another_nested_file.txt");
    d1.create_dir_all()?;
    d2.create_dir_all()?;
    f1.write_str("Some content.\n")?;
    f2.write_str("Some more content.\n")?;
    f3.write_str("Yet more content.\n")?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("write-tree")
        .assert()
        .success();

    std::fs::remove_dir_all(d1.path())?;
    std::fs::remove_file(f1.path())?;

    Command::cargo_bin("rustig")?
        .current_dir(&cwd)
        .arg("read-tree")
        .arg("dcb3fb5c65318b85054a7a687555cbabaf103969")
        .assert()
        .success()
        .stderr(is_empty())
        .stdout(is_empty());

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
