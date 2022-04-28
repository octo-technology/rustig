use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn init_subcommand_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rustig")?;

    cmd.arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Init!"));

    Ok(())
}
