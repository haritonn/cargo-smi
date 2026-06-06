use crate::error::Result;
use nvml_wrapper::{
    Nvml, cuda_driver_version_major, cuda_driver_version_minor,
    enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory,
    struct_wrappers::device::ProcessInfo,
};
use std::collections::VecDeque;
use std::fmt::Display;

// Types

/// Short info about a GPU: its name and NVML index.
#[derive(Debug)]
pub struct GpuDevice {
    /// Human-readable GPU name reported by NVML.
    pub name: String,
    /// NVML device index.
    pub idx: usize,
}

/// All metrics collected for a single GPU.
#[derive(Debug)]
pub struct GpuStats {
    temperature: u32,
    pub utilization: u32,
    memory_used: u64,
    memory_total: u64,
    /// Processes currently using this GPU.
    pub processes: Vec<GpuProcessStats>,
}

/// Runtime entry for a GPU device and its latest stats.
#[derive(Debug)]
pub struct GpuEntry {
    pub device: GpuDevice,
    pub stats: Option<GpuStats>,
    pub util_history: VecDeque<u32>,
}

/// Statistics for a process using GPU resources.
///
/// NVML reports compute and graphics processes separately, so `kind` describes
/// where the process was found after deduplication.
#[derive(Debug)]
pub struct GpuProcessStats {
    /// Operating system process ID.
    pub pid: u32,
    /// Process name resolved through `sysinfo`.
    pub name: String,
    /// GPU memory used by the process, in bytes.
    pub memory: u64,
    /// GPU usage category reported by NVML.
    pub kind: GpuProcKind,
}

/// GPU process category reported by NVML.
///
/// `ComputeAndGraphics` is used when the same PID appears in both NVML process lists.
#[derive(Debug)]
pub enum GpuProcKind {
    Graphics,
    Compute,
    ComputeAndGraphics,
}

/// Core monitoring structure.
pub struct GpuMonitor {
    nvml: Nvml,
}

// Constructors
impl GpuDevice {
    fn new(idx: usize, name: String) -> Self {
        Self { name, idx }
    }
}

impl GpuEntry {
    pub fn new(device: GpuDevice) -> Self {
        Self {
            device,
            stats: None,
            util_history: VecDeque::new(),
        }
    }

    /// Updates statistics for the current `GpuEntry`.
    #[allow(unused)]
    pub fn refresh_stats(&mut self, gpu_monitor: &mut GpuMonitor) -> Result<()> {
        self.stats = Some(gpu_monitor.get_info(self.device.idx, |_| "<unknown>".to_owned())?);
        Ok(())
    }
}

impl GpuStats {
    fn new(
        temperature: u32,
        utilization: u32,
        memory_used: u64,
        memory_total: u64,
        processes: Vec<GpuProcessStats>,
    ) -> Self {
        Self {
            temperature,
            utilization,
            memory_used,
            memory_total,
            processes,
        }
    }
}

impl Display for GpuStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T: {}°C | Util: {}% | Memory: {}/{} MiB",
            self.temperature, self.utilization, self.memory_used, self.memory_total
        )
    }
}

impl Display for GpuProcKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GpuProcKind::Compute => "compute",
            GpuProcKind::Graphics => "graphics",
            GpuProcKind::ComputeAndGraphics => "compute & graphics",
        };
        write!(f, "{output}")
    }
}

// Monitoring
impl GpuMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            nvml: Nvml::init()?,
        })
    }

    /// Returns formatted CUDA and NVIDIA driver versions.
    pub fn cuda_driver_version(&self) -> Result<String> {
        let version = self.nvml.sys_cuda_driver_version()?;
        let driver = self.nvml.sys_driver_version()?;

        Ok(format!(
            "{}.{} | Driver version: {}",
            cuda_driver_version_major(version),
            cuda_driver_version_minor(version),
            driver,
        ))
    }

    /// Returns all available GPU devices.
    pub fn get_available_gpus(&self) -> Result<Vec<GpuDevice>> {
        let device_count = self.nvml.device_count()? as usize;
        let mut gpus = Vec::with_capacity(device_count);

        for idx in 0..device_count {
            let device = self.nvml.device_by_index(idx as u32)?;
            gpus.push(GpuDevice::new(idx, device.name()?));
        }

        Ok(gpus)
    }

    /// Returns GPU stats for the device at `idx`.
    pub fn get_info<F>(&mut self, idx: usize, mut process_name: F) -> Result<GpuStats>
    where
        F: FnMut(u32) -> String,
    {
        let (temperature, utilization, memory_used, memory_total) = {
            let device = self.nvml.device_by_index(idx as u32)?;
            let memory = device.memory_info()?;
            let utilization = device.utilization_rates()?;

            (
                device.temperature(TemperatureSensor::Gpu)?,
                utilization.gpu,
                memory.used / 1024 / 1024,
                memory.total / 1024 / 1024,
            )
        };
        let processes = self.get_gpu_processes(idx, &mut process_name)?;

        Ok(GpuStats::new(
            temperature,
            utilization,
            memory_used,
            memory_total,
            processes,
        ))
    }

    /// Returns GPU processes for the device at `idx`, deduplicated by PID.
    ///
    /// Uses `convert_gpu_process` as helper function.
    pub fn get_gpu_processes<F>(
        &mut self,
        idx: usize,
        process_name: &mut F,
    ) -> Result<Vec<GpuProcessStats>>
    where
        F: FnMut(u32) -> String,
    {
        let device = self.nvml.device_by_index(idx as u32)?;

        let compute = device.running_compute_processes()?;
        let graphics = device.running_graphics_processes()?;

        let mut processes = Vec::with_capacity(compute.len() + graphics.len());

        for process in compute {
            processes.push(convert_gpu_process(
                process,
                GpuProcKind::Compute,
                process_name,
            ));
        }

        for process in graphics {
            let new_process = convert_gpu_process(process, GpuProcKind::Graphics, process_name);

            if let Some(existing) = processes
                .iter_mut()
                .find(|item| item.pid == new_process.pid)
            {
                existing.kind = GpuProcKind::ComputeAndGraphics;
                existing.memory = existing.memory.max(new_process.memory);
            } else {
                processes.push(new_process);
            }
        }

        processes.sort_by(|left, right| right.memory.cmp(&left.memory));
        Ok(processes)
    }
}

/// Converts `nvml_wrapper`'s `ProcessInfo` into `GpuProcessStats`.
fn convert_gpu_process<F>(
    process: ProcessInfo,
    kind: GpuProcKind,
    process_name: &mut F,
) -> GpuProcessStats
where
    F: FnMut(u32) -> String,
{
    let memory = match process.used_gpu_memory {
        UsedGpuMemory::Used(bytes) => bytes,
        UsedGpuMemory::Unavailable => 0,
    };
    let name = process_name(process.pid);

    GpuProcessStats {
        pid: process.pid,
        name,
        memory,
        kind,
    }
}
