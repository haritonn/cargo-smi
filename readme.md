# cargo-smi

Terminal GPU and system monitor written in Rust.

`cargo-smi` shows NVIDIA GPU stats together with basic system information in a single TUI dashboard.

## Features

- NVIDIA GPU detection via `nvidia-smi`
- Live GPU stats:
  - temperature
  - utilization
  - memory usage
- Multiple GPU navigation
- System overview:
  - CPU usage
  - RAM usage
  - swap usage
  - top processes by CPU usage
- Keyboard-driven terminal UI
- Configurable refresh interval

## Requirements

- Rust
- NVIDIA GPU
- NVIDIA driver with `nvidia-smi` available in `$PATH`

## Installation

From source:

```bash
git clone https://github.com/haritonn/cargo-smi
cd cargo_smi
cargo build --release
```

Run:

```bash
cargo run --release
```

Or with custom refresh interval in seconds:

```bash
cargo run --release -- 2
```
## Known limitations

- NVIDIA-only for now
- Requires nvidia-smi
- Process list is sorted by CPU usage
- First CPU readings may need one refresh cycle to stabilize
