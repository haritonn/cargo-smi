use std::process::ExitStatus;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CargoSmiError>;

#[derive(Debug, Error)]
pub enum CargoSmiError {
    #[error("failed to start nvidia-smi: {0}")]
    Command(#[from] std::io::Error),

    #[error("nvidia-smi failed with status {status}: {stderr}")]
    NvidiaSmiFailed { status: ExitStatus, stderr: String },

    #[error("nvidia-smi output is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("failed to parse numeric value from nvidia-smi output: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(
        "unexpected nvidia-smi output: expected at least {expected} columns, got {got}: {raw:?}"
    )]
    InvalidOutput {
        expected: usize,
        got: usize,
        raw: String,
    },
}

impl CargoSmiError {
    pub fn invalid_output_len(expected: usize, got: usize, raw: &str) -> Self {
        CargoSmiError::InvalidOutput {
            expected,
            got,
            raw: raw.to_owned(),
        }
    }
}
