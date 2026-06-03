# cargo-smi

<p align="center">
  <img src="assets/app.png" alt="cargo-smi TUI dashboard" width="900">
</p>

<p align="center">
  <strong>A fast terminal dashboard for NVIDIA GPU and system monitoring.</strong>
</p>

---

`cargo-smi` is a lightweight TUI monitor that shows NVIDIA GPU metrics, GPU processes, and basic system information in one terminal dashboard.

Unlike shelling out to `nvidia-smi`, GPU data is collected directly through NVIDIA Management Library using [`nvml-wrapper`](https://docs.rs/nvml-wrapper), avoiding command output parsing.

## Features

- Direct NVIDIA GPU detection via NVML
- Live GPU stats:
  - temperature
  - utilization
  - memory usage
- GPU process list:
  - PID
  - process name
  - used GPU memory
  - compute / graphics process kind
- Multiple GPU navigation
- CUDA driver version and NVIDIA driver version display
- System overview:
  - CPU usage
  - RAM usage
  - swap usage
  - top processes by CPU usage
- Keyboard-driven terminal UI
- Configurable refresh interval in milliseconds

## Requirements

- Rust
- NVIDIA GPU
- NVIDIA driver with NVML available

On most Linux systems with the proprietary NVIDIA driver installed, NVML is available as `libnvidia-ml.so`.

## Installation

Clone and build from source:

```bash
git clone https://github.com/haritonn/cargo-smi
cd cargo-smi
cargo build --release
```

Run from source:

```bash
cargo run --release
```

Install locally with Cargo:

```bash
cargo install --path .
```

Then run:

```bash
cargo_smi
```

## Usage

Run with default refresh interval:

```bash
cargo run --release
```

Run with a custom refresh interval in milliseconds:

```bash
cargo run --release -- 500
```

## Known limitations

- NVIDIA-only for now
- Requires NVIDIA driver / NVML
- GPU process utilization percentage is not shown
- First CPU readings may need one refresh cycle to stabilize
