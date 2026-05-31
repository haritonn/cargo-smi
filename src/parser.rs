use std::{
    error::Error,
    process::{Command, Output},
    result::Result,
};

pub fn get_info() -> Result<Output, Box<dyn Error>> {
    let res = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,power.draw,power.limit,fan.speed",
            "--format=csv,noheader,nounits",
        ])
        .output()?;
    Ok(res)
}
