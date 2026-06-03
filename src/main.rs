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

fn main() {
    if let Err(err) = run_main() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[allow(unreachable_code)]
fn run_main() -> Result<()> {
    let sleep_arg = args().nth(1).unwrap_or_else(|| "1".to_owned());
    let sleep_secs = sleep_arg
        .parse::<u64>()
        .map_err(|_| CargoSmiError::CliArg {
            arg: sleep_arg.clone(),
        })?;

    let gpu_monitor = GpuMonitor::new()?;
    let gpus = gpu_monitor.get_available_gpus()?;
    if gpus.is_empty() {
        return Err(CargoSmiError::NoGpuFound);
    }

    let mut state = AppState::new(gpus, gpu_monitor, Duration::from_secs(sleep_secs))?;
    crate::ui::run_tui(&mut state)?;
    Ok(())
}
