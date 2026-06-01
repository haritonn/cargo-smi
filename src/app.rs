use crate::{
    error,
    parser::{GpuDevice, GpuEntry},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct AppState {
    gpus: HashMap<usize, GpuEntry>,
    gpu_order: Vec<usize>,
    selected_pos: Option<usize>,
    should_quit: bool,
    last_error: Option<String>,
    interval: Duration,
    last_update: Instant,
}

impl AppState {
    pub fn new(gpus: Vec<GpuDevice>, interval: Duration) -> Self {
        let gpu_order: Vec<usize> = gpus.iter().map(|gpu| gpu.idx).collect();
        let selected_pos = if gpu_order.is_empty() { None } else { Some(0) };

        Self {
            gpus: gpus
                .into_iter()
                .map(|gpu| (gpu.idx, GpuEntry::new(gpu)))
                .collect(),
            gpu_order,
            selected_pos,
            should_quit: false,
            last_error: None,
            interval,
            last_update: Instant::now(),
        }
    }

    #[allow(dead_code)]
    pub fn select_gpu(&mut self, idx: usize) -> error::Result<()> {
        let pos = self
            .gpu_order
            .iter()
            .position(|gpu_idx| *gpu_idx == idx)
            .ok_or(error::CargoSmiError::GpuNotFound { idx })?;

        self.selected_pos = Some(pos);
        Ok(())
    }

    pub fn select_next(&mut self) {
        if self.gpu_order.is_empty() {
            self.selected_pos = None;
            return;
        }

        self.selected_pos = Some(match self.selected_pos {
            Some(pos) => (pos + 1) % self.gpu_order.len(),
            None => 0,
        });
    }

    pub fn select_prev(&mut self) {
        if self.gpu_order.is_empty() {
            self.selected_pos = None;
            return;
        }

        self.selected_pos = Some(match self.selected_pos {
            Some(0) | None => self.gpu_order.len() - 1,
            Some(pos) => pos - 1,
        });
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    fn selected_idx(&self) -> error::Result<usize> {
        let pos = self
            .selected_pos
            .ok_or(error::CargoSmiError::NoGpuSelected)?;

        self.gpu_order
            .get(pos)
            .copied()
            .ok_or(error::CargoSmiError::NoGpuSelected)
    }

    pub fn selected_gpu_mut(&mut self) -> error::Result<&mut GpuEntry> {
        let idx = self.selected_idx()?;
        self.gpus
            .get_mut(&idx)
            .ok_or(error::CargoSmiError::GpuNotFound { idx })
    }

    pub fn refresh_selected(&mut self) {
        match self.selected_gpu_mut().and_then(GpuEntry::refresh_stats) {
            Ok(()) => self.last_error = None,
            Err(err) => self.last_error = Some(err.to_string()),
        }
        self.last_update = Instant::now();
    }

    pub fn should_refresh(&self) -> bool {
        self.last_update.elapsed() >= self.interval
    }

    pub fn gpu_entries(&self) -> Vec<&GpuEntry> {
        self.gpu_order
            .iter()
            .filter_map(|idx| self.gpus.get(idx))
            .collect()
    }
    pub fn selected_pos(&self) -> Option<usize> {
        self.selected_pos
    }
}
