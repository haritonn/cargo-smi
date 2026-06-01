use crate::error::{CargoSmiError, Result};
use std::fmt::Display;
use std::process::{Command, Output};

#[derive(Debug)]
pub struct GpuDevice {
    pub name: String,
    pub idx: usize,
}

#[derive(Debug)]
pub struct GpuStats {
    temperature: i16,
    utilization: u8,
    memory_used: u32,
    memory_total: u32,
}

#[derive(Debug)]
pub struct GpuEntry {
    pub device: GpuDevice,
    pub stats: Option<GpuStats>,
}

impl GpuStats {
    fn new(temperature: i16, utilization: u8, memory_used: u32, memory_total: u32) -> Self {
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

    pub fn refresh_stats(&mut self) -> Result<()> {
        self.stats = Some(self.device.get_info()?);
        Ok(())
    }
}

impl GpuDevice {
    fn new(idx: usize, name: String) -> Self {
        Self { name, idx }
    }

    fn parse_stats(&self, stdout_string: &str) -> Result<GpuStats> {
        let res_items: Vec<&str> = stdout_string.trim().split(',').map(str::trim).collect();
        if res_items.len() < 4 {
            return Err(CargoSmiError::invalid_output_len(
                4,
                res_items.len(),
                stdout_string,
            ));
        }

        Ok(GpuStats::new(
            res_items[0].parse::<i16>()?,
            res_items[1].parse::<u8>()?,
            res_items[2].parse::<u32>()?,
            res_items[3].parse::<u32>()?,
        ))
    }

    pub fn get_info(&self) -> Result<GpuStats> {
        let cmd_res = Command::new("nvidia-smi")
            .args([
                "-i",
                &self.idx.to_string(),
                "--query-gpu=temperature.gpu,utilization.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()?;

        let stdout_string = validate_cmd(cmd_res)?;
        self.parse_stats(&stdout_string)
    }
}

pub fn get_available_gpus() -> Result<Vec<GpuDevice>> {
    let cmd_res = Command::new("nvidia-smi")
        .args(["--query-gpu=index,name", "--format=csv,noheader,nounits"])
        .output()?;
    let stdout_string = validate_cmd(cmd_res)?;

    stdout_string
        .lines()
        .map(|line| {
            let items: Vec<&str> = line.splitn(2, ',').map(str::trim).collect();
            if items.len() < 2 {
                return Err(CargoSmiError::invalid_output_len(2, items.len(), line));
            }

            Ok(GpuDevice::new(
                items[0].parse::<usize>()?,
                items[1].to_owned(),
            ))
        })
        .collect()
}

fn validate_cmd(cmd_res: Output) -> Result<String> {
    if !cmd_res.status.success() {
        return Err(CargoSmiError::NvidiaSmiFailed {
            status: cmd_res.status,
            stderr: String::from_utf8_lossy(&cmd_res.stderr).into_owned(),
        });
    }
    Ok(String::from_utf8(cmd_res.stdout)?)
}
