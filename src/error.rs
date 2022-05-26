use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid type (expected {expected:?}, got {found:?})")]
    InvalidType { expected: String, found: String },
}
