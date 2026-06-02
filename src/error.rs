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

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("NVML error: {0}")]
    Nvml(#[from] nvml_wrapper::error::NvmlError),
}
