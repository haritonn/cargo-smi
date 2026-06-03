//! Error types used across the application.

use thiserror::Error;

/// Convenient result alias for cargo-smi operations.
pub type Result<T> = std::result::Result<T, CargoSmiError>;

/// Application-level error type.
#[derive(Debug, Error)]
pub enum CargoSmiError {
    /// Invalid command-line argument.
    #[error("CLI error occurred: {arg}")]
    CliArg { arg: String },

    /// No NVIDIA GPU was detected by NVML.
    #[error("No GPU found at all")]
    NoGpuFound,

    /// UI state does not contain a selected GPU.
    #[error("No GPU selected")]
    NoGpuSelected,

    /// GPU index was present in UI state but missing from the GPU map.
    #[error("GPU with index {idx} not found")]
    GpuNotFound { idx: usize },

    /// Terminal or other I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error returned by NVML.
    #[error("NVML error: {0}")]
    Nvml(#[from] nvml_wrapper::error::NvmlError),
}
