//! Entry point for the cargo-smi terminal application.

mod app;
mod error;
mod gpu;
mod system;
mod ui;

use crate::{
    app::AppState,
    error::{CargoSmiError, Result},
    gpu::GpuMonitor,
};
use std::{env::args, time::Duration};

/// Runs the application and exits with a non-zero status on error.
fn main() {
    if let Err(err) = run_main() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

/// Parses CLI options, initializes monitors, and starts the TUI.
#[allow(unreachable_code)]
fn run_main() -> Result<()> {
    let sleep_arg = args().nth(1).unwrap_or_else(|| "500".to_owned());
    let sleep_millis = sleep_arg
        .parse::<u64>()
        .map_err(|_| CargoSmiError::CliArg {
            arg: sleep_arg.clone(),
        })?;

    let gpu_monitor = GpuMonitor::new()?;
    let gpus = gpu_monitor.get_available_gpus()?;
    if gpus.is_empty() {
        return Err(CargoSmiError::NoGpuFound);
    }

    let mut state = AppState::new(gpus, gpu_monitor, Duration::from_millis(sleep_millis))?;
    crate::ui::run_tui(&mut state)?;
    Ok(())
}
