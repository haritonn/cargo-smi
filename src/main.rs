mod app;
mod error;
mod parser;

use crate::{
    app::{AppState, run},
    error::{CargoSmiError, Result},
    parser::get_available_gpus,
};
use std::{env::args, io::stdin};

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

    let gpus = get_available_gpus()?;
    if gpus.is_empty() {
        return Err(CargoSmiError::NoGpuFound);
    }
    gpus.iter()
        .for_each(|gpu| println!("{}: {}", gpu.idx, gpu.name));

    let mut choice_string = String::new();
    println!("Enter your choice: ");
    stdin().read_line(&mut choice_string)?;

    let choice_arg = choice_string.trim();
    let choice: usize = choice_arg.parse().map_err(|_| CargoSmiError::CliArg {
        arg: choice_arg.to_owned(),
    })?;
    let mut state = AppState::new(gpus);
    state.select_gpu(choice);
    run(&mut state, sleep_secs)?;
    Ok(())
}
