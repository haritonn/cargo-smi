use std::{num::ParseIntError, process::ExitStatus};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CargoSmiError>;

#[derive(Debug, Error)]
pub enum CargoSmiError {
    #[error("CLI error occured: {arg}")]
    CliArg { arg: String },

    #[error("No GPU found at all")]
    NoGpuFound,

    #[error("No GPU selected")]
    NoGpuSelected,

    #[error("GPU with index {idx} not found")]
    GpuNotFound { idx: usize },

    #[error("Failed to start nvidia-smi: {0}")]
    Command(#[from] std::io::Error),

    #[error("nvidia-smi failed with status {status}: {stderr}")]
    NvidiaSmiFailed { status: ExitStatus, stderr: String },

    #[error("nvidia-smi output is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Failed to parse field `{field}` from value {value:?}: {source}")]
    ParseNumber {
        field: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },

    #[error(
        "Unexpected nvidia-smi output: expected at least {expected} columns, got {got}: {raw:?}"
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
