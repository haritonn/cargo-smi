//! System metrics collection via `sysinfo`.
//!
//! This module tracks CPU, memory, swap, and top CPU-consuming processes for
//! display in the TUI.

use std::cmp::Ordering;
use sysinfo::{Components, Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Snapshot of system-wide metrics.
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// Global CPU usage as reported by `sysinfo`, in percent.
    pub cpu_usage: f32,
    /// CPU temperature in Celcius.
    ///
    /// This can be not available due to hardware issues,
    /// lack of permissions or something else.
    pub cpu_temp: Option<f32>,
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

/// Picks the CPU package/die temperature out of the sensor list if possible.
///
/// There is no single "CPU temperature" sensor exposed by the OS, so this
/// matches on the label conventions used by the common Linux `hwmon`
/// drivers.
///
/// Intel uses `package id 0` label, AMD uses `tctl` and `tdie`.
fn cpu_temperature(components: &Components) -> Option<f32> {
    let labels: &[&str] = &["package id 0", "tctl", "tdie"];

    for label in labels {
        if let Some(temp) = components.iter().find_map(|c| {
            c.label()
                .to_lowercase()
                .contains(label)
                .then(|| c.temperature())
                .flatten()
        }) {
            return Some(temp);
        }
    }

    None
}

/// Stateful system monitor backed by `sysinfo::System`.
pub struct SystemMonitor {
    system: System,
    components: Components,
    process_limit: usize,
}

impl SystemMonitor {
    /// Creates a system monitor that keeps at most `process_limit` processes.
    pub fn new(process_limit: usize) -> Self {
        let mut monitor = Self {
            system: System::new(),
            components: Components::new_with_refreshed_list(),
            process_limit,
        };
        monitor.refresh();
        monitor
    }

    /// Returns a cached process name, if the process is known to `sysinfo`.
    pub fn process_name(&self, pid: u32) -> String {
        self.system
            .process(Pid::from_u32(pid))
            .map(|process| process.name().to_str().unwrap_or("<non-utf8>").to_owned())
            .unwrap_or_else(|| "<unknown>".to_owned())
    }

    /// Refreshes system data and returns a sorted metrics snapshot.
    pub fn refresh(&mut self) -> SystemStats {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.components.refresh(false);
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .without_tasks(),
        );

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
            cpu_temp: cpu_temperature(&self.components),
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
