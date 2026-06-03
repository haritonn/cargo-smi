use crate::error::Result;
use nvml_wrapper::{
    Nvml, cuda_driver_version_major, cuda_driver_version_minor,
    enum_wrappers::device::TemperatureSensor, enums::device::UsedGpuMemory,
    struct_wrappers::device::ProcessInfo,
};
use std::fmt::Display;
use sysinfo::{Pid, System};

#[derive(Debug)]
pub enum GpuProcKind {
    Graphics,
    Compute,
    ComputeAndGraphics,
}

impl Display for GpuProcKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            GpuProcKind::Compute => "compute",
            GpuProcKind::Graphics => "graphics",
            _ => "compute & graphics",
        };
        write!(f, "{output}")
    }
}

#[derive(Debug)]
pub struct GpuDevice {
    pub name: String,
    pub idx: usize,
}

#[derive(Debug)]
pub struct GpuStats {
    temperature: u32,
    utilization: u32,
    memory_used: u64,
    memory_total: u64,
    pub processes: Vec<GpuProcessStats>,
}

#[derive(Debug)]
pub struct GpuProcessStats {
    pub pid: u32,
    pub name: String,
    // pub gpu_usage: f32,
    pub memory: u64,
    pub kind: GpuProcKind,
}

#[derive(Debug)]
pub struct GpuEntry {
    pub device: GpuDevice,
    pub stats: Option<GpuStats>,
}

pub struct GpuMonitor {
    nvml: Nvml,
    system: System,
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

impl GpuEntry {
    pub fn new(device: GpuDevice) -> Self {
        Self {
            device,
            stats: None,
        }
    }

    #[allow(unused)]
    pub fn refresh_stats(&mut self, gpu_monitor: &mut GpuMonitor) -> Result<()> {
        self.stats = Some(gpu_monitor.get_info(self.device.idx)?);
        Ok(())
    }
}

impl GpuDevice {
    fn new(idx: usize, name: String) -> Self {
        Self { name, idx }
    }
}

impl GpuMonitor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            nvml: Nvml::init()?,
            system: System::new_all(),
        })
    }

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

    pub fn get_available_gpus(&self) -> Result<Vec<GpuDevice>> {
        let device_count = self.nvml.device_count()? as usize;
        let mut gpus = Vec::with_capacity(device_count);

        for idx in 0..device_count {
            let device = self.nvml.device_by_index(idx as u32)?;
            gpus.push(GpuDevice::new(idx, device.name()?));
        }

        Ok(gpus)
    }

    pub fn get_info(&mut self, idx: usize) -> Result<GpuStats> {
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
        let processes = self.get_gpu_processes(idx)?;

        Ok(GpuStats::new(
            temperature,
            utilization,
            memory_used,
            memory_total,
            processes,
        ))
    }

    pub fn get_gpu_processes(&mut self, idx: usize) -> Result<Vec<GpuProcessStats>> {
        let device = self.nvml.device_by_index(idx as u32)?;
        self.system.refresh_all();

        let compute = device.running_compute_processes()?;
        let graphics = device.running_graphics_processes()?;

        let mut processes = vec![];
        for proc in compute {
            let new_proc = convert_gpu_process(proc, GpuProcKind::Compute, &self.system);
            processes.push(new_proc);
        }
        for proc in graphics {
            let new_proc = convert_gpu_process(proc, GpuProcKind::Graphics, &self.system);
            if let Some(existing) = processes.iter_mut().find(|x| x.pid == new_proc.pid) {
                existing.kind = GpuProcKind::ComputeAndGraphics;
                existing.memory = existing.memory.max(new_proc.memory);
            } else {
                processes.push(new_proc);
            }
        }

        processes.sort_by(|left, right| right.memory.cmp(&left.memory));
        Ok(processes)
    }
}

fn convert_gpu_process(
    process: ProcessInfo,
    kind: GpuProcKind,
    system: &System,
) -> GpuProcessStats {
    let memory = match process.used_gpu_memory {
        UsedGpuMemory::Used(bytes) => bytes,
        UsedGpuMemory::Unavailable => 0,
    };
    let name = system
        .process(Pid::from_u32(process.pid))
        .map(|process| process.name().to_str().unwrap_or("<non-utf8>").to_owned())
        .unwrap_or_else(|| "<unknown>".to_owned());

    GpuProcessStats {
        pid: process.pid,
        name,
        memory,
        kind,
    }
}
