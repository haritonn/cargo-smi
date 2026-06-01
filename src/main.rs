mod error;
mod parser;

use crate::parser::*;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::{
    collections::HashMap,
    env::args,
    io::{Write, stdin, stdout},
    thread,
    time::Duration,
};

pub struct AppState {
    gpus: HashMap<usize, GpuEntry>,
    selected_idx: Option<usize>,
}

impl AppState {
    fn new(gpus: Vec<GpuDevice>) -> Self {
        Self {
            gpus: gpus
                .into_iter()
                .map(|gpu| (gpu.idx, GpuEntry::new(gpu)))
                .collect(),
            selected_idx: None,
        }
    }

    fn select_gpu(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
    }

    fn selected_gpu_mut(&mut self) -> Option<&mut GpuEntry> {
        let idx = self.selected_idx?;
        self.gpus.get_mut(&idx)
    }
}

fn run(state: &mut AppState, sleep_secs: u64) -> error::Result<()> {
    let mut stdout = stdout();
    loop {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        let entry = state.selected_gpu_mut().expect("bad GPU index");
        entry.refresh_stats()?;
        if let Some(stats) = &entry.stats {
            println!("{} | {}", entry.device.name, stats);
        }
        stdout.flush()?;
        thread::sleep(Duration::from_secs(sleep_secs));
    }
}

#[allow(unreachable_code)]
fn main() -> error::Result<()> {
    let sleep_secs: u64 = args().nth(1).as_deref().unwrap_or("1").parse()?;

    let gpus = get_available_gpus()?;
    gpus.iter()
        .for_each(|gpu| println!("{}: {}", gpu.idx, gpu.name));

    let mut choice_string = String::new();
    println!("Enter your choice: ");
    stdin().read_line(&mut choice_string)?;

    let choice: usize = choice_string.trim().parse().expect("bad choice");
    let mut state = AppState::new(gpus);
    state.select_gpu(choice);
    run(&mut state, sleep_secs)?;
    Ok(())
}
