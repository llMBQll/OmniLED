# Installing on Windows

1. [Install dependencies](#install-dependencies)
2. [Build from source](#build-from-source) or install using [prebuilt binaries](#prebuilt-binaries).

## Install Dependencies

### Linux

- `gcc` - `apt install gcc`
- `g++` - `apt install g++`
- `libdbus` - `apt install libdbus-1-dev`
- `libfontconfig` - `apt install libfontconfig-dev`
- `libx11` - `apt install libx11-dev`
- `pkg-config` - `apt install pkg-config`
- `protobuf-compiler` - `apt install protobuf-compiler`

> _Note: Tested on Ubuntu 24.04 LTS_

### Windows

- [SteelSeries Engine](https://steelseries.com/gg/engine) - Optional. All devices can be controlled
  via raw USB calls, routing via SSE is optional.

### Common

- [Rust](https://rustup.rs/) - Optional. Only required when building from source.
- [Cargo Make](https://crates.io/crates/cargo-make#installation) - Optional. This is convenient
  when building from source, though the commands may also be written by hand.

## Build from Source

1. Open Terminal.
2. Download repository and go into the directory.
   > `git clone https://github.com/llMBQll/OmniLED`  
   > `cd OmniLED`
3. Build & Install
   If you have installed cargo make (see [dependencies](#install-dependencies)) you may execute a
   single command to build binaries and run setup utility.
   > `cargo make run-setup`

   Else you need to compile the targets manually. This requires 2 build steps due to package
   dependencies.
   > `cargo build --release -p omni-led -p audio -p clock -p media -p weather`  
   > `cargo build --release -p setup`  
   > `cargo run --release --bin omni-led-setup -- install --interactive`

## Post installation steps

### Linux

#### Allow USB access

To allow this program to access your device, it needs an entry in udev rules.

1. Create udev rules entry.  
   `touch /etc/udev/rules.d/69-omni-led.rules`
2. Using your favourite text editor add the following line and adapt it for your device.  
   `SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1618", MODE="0666", GROUP="plugdev"`
3. Reload udev rules (this may not be sufficient so you may need to unplug and plug the device or
   restart the system).  
   `sudo udevadm control --reload-rules`

### Common

You are now ready to proceed to [customization](customization).

## Prebuilt Binaries

You can find prebuilt binaries for Windows in the
[releases](https://github.com/llMBQll/OmniLED/releases) section on GitHub.
