# Installing on Windows

1. [Install dependencies](#install-dependencies)
2. [Build from source](#build-from-source) or install using [prebuilt binaries](#prebuilt-binaries).

## Install Dependencies

- [Rust](https://rustup.rs/) - Optional. Only required when building from source.
- [SteelSeries Engine](https://steelseries.com/gg/engine) - Optional. By default, all devices are controlled via raw USB
  calls, routing via SSE is optional.

## Build from Source

1. Open Terminal
2. Download repository and go into the directory  
   `git clone TODO: URL`  
   `cd TODO: name`
3. Build  
   `cargo build --release`
4. Install  
   `cargo run --bin setup -- --dev`

## Prebuilt Binaries

Soon™️