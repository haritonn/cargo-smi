//! System metrics collection via `sysinfo`.
//!
//! This module tracks CPU, memory, swap, and top CPU-consuming processes for
//! display in the TUI.

use std::cmp::Ordering;
use sysinfo::System;

/// Snapshot of system-wide metrics.
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// Global CPU usage as reported by `sysinfo`, in percent.
    pub cpu_usage: f32,
    /// Used RAM in bytes.
    pub memory_used: u64,
    /// Total RAM in bytes.
    pub memory_total: u64,
    /// Used swap in bytes.
    pub swap_used: u64,
    /// Total swap in bytes.
    pub swap_total: u64,
    /// Top processes sorted by CPU usage.
    pub processes: Vec<ProcessStats>,
}

/// Snapshot of a single operating system process.
#[derive(Debug, Clone)]
pub struct ProcessStats {
    /// Operating system process ID.
    pub pid: String,
    /// Process name.
    pub name: String,
    /// Process CPU usage, in percent.
    ///
    /// This can exceed 100% when a process uses multiple CPU cores.
    pub cpu_usage: f32,
    /// Resident memory used by the process, in bytes.
    pub memory: u64,
}

/// Stateful system monitor backed by `sysinfo::System`.
pub struct SystemMonitor {
    system: System,
    process_limit: usize,
}

impl SystemMonitor {
    /// Creates a system monitor that keeps at most `process_limit` processes.
    pub fn new(process_limit: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            process_limit,
        }
    }

    /// Refreshes system data and returns a sorted metrics snapshot.
    pub fn refresh(&mut self) -> SystemStats {
        // self.system.refresh_cpu_usage();
        // self.system.refresh_memory();
        // self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.system.refresh_all();

        let mut processes: Vec<ProcessStats> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessStats {
                pid: pid.to_string(),
                name: process.name().to_str().unwrap_or("<non-utf8>").to_owned(),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            })
            .collect();

        processes.sort_by(|left, right| {
            right
                .cpu_usage
                .partial_cmp(&left.cpu_usage)
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.memory.cmp(&left.memory))
        });
        processes.truncate(self.process_limit);

        SystemStats {
            cpu_usage: self.system.global_cpu_usage(),
            memory_used: self.system.used_memory(),
            memory_total: self.system.total_memory(),
            swap_used: self.system.used_swap(),
            swap_total: self.system.total_swap(),
            processes,
        }
    }
}

impl Default for SystemMonitor {
    /// Creates a monitor showing the top 20 processes.
    fn default() -> Self {
        Self::new(20)
    }
}
