use crate::error::Result;
use nvml_wrapper::{
    Nvml, cuda_driver_version_major, cuda_driver_version_minor,
    enum_wrappers::device::TemperatureSensor,
};
use std::fmt::Display;

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
}

#[derive(Debug)]
pub struct GpuEntry {
    pub device: GpuDevice,
    pub stats: Option<GpuStats>,
}

pub struct GpuMonitor {
    nvml: Nvml,
}

impl GpuStats {
    fn new(temperature: u32, utilization: u32, memory_used: u64, memory_total: u64) -> Self {
        Self {
            temperature,
            utilization,
            memory_used,
            memory_total,
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
    pub fn refresh_stats(&mut self, gpu_monitor: &GpuMonitor) -> Result<()> {
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

    pub fn get_info(&self, idx: usize) -> Result<GpuStats> {
        let device = self.nvml.device_by_index(idx as u32)?;
        let memory = device.memory_info()?;
        let utilization = device.utilization_rates()?;

        Ok(GpuStats::new(
            device.temperature(TemperatureSensor::Gpu)?,
            utilization.gpu,
            memory.used / 1024 / 1024,
            memory.total / 1024 / 1024,
        ))
    }

    fn get_gpu_processes(&self) {
        unimplemented!()
    }
}
