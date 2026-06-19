# Installing

## Prebuilt binaries

You can find prebuilt binaries in the [releases](https://github.com/llMBQll/OmniLED/releases) section on GitHub.  
Download the binry for your platform and install the application, then see
[post installation steps](#post-installation-steps).

## Build from source

### Install Dependencies

#### Common

- [Rust](https://rustup.rs/)
- [Cargo Packager](https://crates.io/crates/cargo-packager)

#### Linux

```bash
sudo apt install \
          gcc \
          g++ \
          libappindicator3-dev \
          libdbus-1-dev \
          libdrm-dev \
          librsvg2-dev \
          libfontconfig-dev \
          libgtk-3-dev \
          libpulse-dev \
          libudev-dev \
          libx11-dev \
          libxdo-dev \
          pkg-config \
          protobuf-compiler
```

> _Note: Tested on Ubuntu 24.04 LTS_

### Compile

1. Open Terminal
2. Download repository and go into the directory
   > `git clone https://github.com/llMBQll/OmniLED`  
   > `cd OmniLED`
3. Build
   > `cargo packager --release --formats appimage # for Linux`  
   > `cargo packager --release --formats dmg      # for macOS`  
   > `cargo packager --release --formats wix      # for Windows`
4. Run the created installer

## Post installation steps

### Linux

#### Allow USB access

To allow this program to access your device, it needs an entry in udev rules.

1. Create udev rules entry.  
   `touch /etc/udev/rules.d/69-omni-led.rules`
2. Using your favourite text editor add the following lines and adapt it for your device  

   ```text
   SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1618", MODE="0666", GROUP="plugdev"
   KERNEL=="hidraw*", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1618", MODE="0666", GROUP="plugdev"
   ```

3. Reload udev rules (in case this is insufficient, you may need to unplug and plug in the device or
   restart the system).  
   `sudo udevadm control --reload-rules && sudo udevadm trigger`

### Common

You are now ready to proceed to [customization](customization).
