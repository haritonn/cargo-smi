use std::cmp::Ordering;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub processes: Vec<ProcessStats>,
}

#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub pid: String,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

pub struct SystemMonitor {
    system: System,
    process_limit: usize,
}

impl SystemMonitor {
    pub fn new(process_limit: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            process_limit,
        }
    }

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
    fn default() -> Self {
        Self::new(20)
    }
}
