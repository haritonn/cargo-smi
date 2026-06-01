use crate::{
    error,
    parser::{GpuDevice, GpuEntry},
};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::{
    collections::HashMap,
    io::{Write, stdout},
    thread,
    time::Duration,
};

pub struct AppState {
    gpus: HashMap<usize, GpuEntry>,
    selected_idx: Option<usize>,
}

impl AppState {
    pub fn new(gpus: Vec<GpuDevice>) -> Self {
        Self {
            gpus: gpus
                .into_iter()
                .map(|gpu| (gpu.idx, GpuEntry::new(gpu)))
                .collect(),
            selected_idx: None,
        }
    }

    pub fn select_gpu(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
    }

    fn selected_gpu_mut(&mut self) -> error::Result<&mut GpuEntry> {
        let idx = self
            .selected_idx
            .ok_or(error::CargoSmiError::NoGpuSelected)?;

        self.gpus
            .get_mut(&idx)
            .ok_or(error::CargoSmiError::GpuNotFound { idx })
    }
}

pub fn run(state: &mut AppState, sleep_secs: u64) -> error::Result<()> {
    let mut stdout = stdout();
    loop {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        let entry = state.selected_gpu_mut()?;
        entry.refresh_stats()?;
        if let Some(stats) = &entry.stats {
            println!("{} | {}", entry.device.name, stats);
        }
        stdout.flush()?;
        thread::sleep(Duration::from_secs(sleep_secs));
    }
}
