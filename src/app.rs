//! Application state and refresh orchestration.
//!
//! This module keeps UI-independent runtime state: selected GPU, cached metrics,
//! refresh timing, and recoverable errors shown in the TUI.

use crate::{
    error,
    gpu::{GpuDevice, GpuEntry, GpuMonitor},
    system::{SystemMonitor, SystemStats},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const UTIL_LIMIT: usize = 120;

/// Runtime state shared by the TUI and monitoring code.
pub struct AppState {
    gpus: HashMap<usize, GpuEntry>,
    gpu_order: Vec<usize>,
    selected_pos: Option<usize>,
    should_quit: bool,
    last_error: Option<String>,
    interval: Duration,
    last_update: Instant,
    system_monitor: SystemMonitor,
    system_stats: Option<SystemStats>,
    gpu_monitor: GpuMonitor,
    cuda_version: String,
}

impl AppState {
    /// Creates application state from detected GPUs and monitor instances.
    pub fn new(
        gpus: Vec<GpuDevice>,
        gpu_monitor: GpuMonitor,
        interval: Duration,
    ) -> error::Result<Self> {
        let gpu_order: Vec<usize> = gpus.iter().map(|gpu| gpu.idx).collect();
        let selected_pos = if gpu_order.is_empty() { None } else { Some(0) };
        let cuda_version = gpu_monitor.cuda_driver_version()?;

        Ok(Self {
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
            system_monitor: SystemMonitor::default(),
            system_stats: None,
            gpu_monitor,
            cuda_version,
        })
    }

    /// Selects the next GPU, wrapping around at the end.
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

    /// Selects the previous GPU, wrapping around at the beginning.
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

    /// Requests application shutdown.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Returns whether the application should exit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Returns the latest recoverable refresh error, if any.
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

    /// Returns a mutable entry for the currently selected GPU.
    #[allow(unused)]
    pub fn selected_gpu_mut(&mut self) -> error::Result<&mut GpuEntry> {
        let idx = self.selected_idx()?;
        self.gpus
            .get_mut(&idx)
            .ok_or(error::CargoSmiError::GpuNotFound { idx })
    }

    /// Returns the entry for the currently selected GPU.
    pub fn selected_gpu(&self) -> error::Result<&GpuEntry> {
        let idx = self.selected_idx()?;
        self.gpus
            .get(&idx)
            .ok_or(error::CargoSmiError::GpuNotFound { idx })
    }

    /// Refreshes metrics for the currently selected GPU.
    ///
    /// Errors are stored in `last_error` instead of being propagated so the UI
    /// can keep running after transient NVML failures.
    pub fn refresh_selected(&mut self) {
        let refresh_result = match self.selected_idx() {
            Ok(idx) => {
                let system_monitor = &self.system_monitor;
                self.gpu_monitor
                    .get_info(idx, |pid| system_monitor.process_name(pid))
                    .map(|stats| (idx, stats))
            }
            Err(err) => Err(err),
        };

        match refresh_result {
            Ok((idx, stats)) => {
                if let Some(gpu) = self.gpus.get_mut(&idx) {
                    let util = stats.utilization;
                    gpu.util_history.push_back(util);
                    if gpu.util_history.len() > UTIL_LIMIT {
                        gpu.util_history.pop_front();
                    }
                    gpu.stats = Some(stats);
                    self.last_error = None;
                } else {
                    self.last_error = Some(error::CargoSmiError::GpuNotFound { idx }.to_string());
                }
            }
            Err(err) => self.last_error = Some(err.to_string()),
        }
        self.last_update = Instant::now();
    }

    /// Refreshes cached system metrics.
    pub fn refresh_system(&mut self) {
        self.system_stats = Some(self.system_monitor.refresh());
    }
    /// Returns latest cached system metrics.
    pub fn system_stats(&self) -> Option<&SystemStats> {
        self.system_stats.as_ref()
    }

    /// Returns formatted CUDA and NVIDIA driver version string.
    pub fn cuda_version(&self) -> &str {
        &self.cuda_version
    }

    /// Refreshes both selected GPU metrics and system metrics.
    pub fn refresh_all(&mut self) {
        self.refresh_system();
        self.refresh_selected();
    }

    /// Returns whether the configured refresh interval has elapsed.
    pub fn should_refresh(&self) -> bool {
        self.last_update.elapsed() >= self.interval
    }

    /// Returns how long the UI can sleep before the next scheduled refresh.
    pub fn time_until_next_refresh(&self) -> Duration {
        self.interval.saturating_sub(self.last_update.elapsed())
    }

    /// Returns GPU entries in stable display order.
    pub fn gpu_entries(&self) -> Vec<&GpuEntry> {
        self.gpu_order
            .iter()
            .filter_map(|idx| self.gpus.get(idx))
            .collect()
    }
    /// Returns selected GPU position in display order.
    pub fn selected_pos(&self) -> Option<usize> {
        self.selected_pos
    }
}
