# Installing on Linux

1. [Install dependencies](#install-dependencies)
2. [Allow USB access](#allow-usb-access)
3. [Build from source](#build-from-source)

## Install Dependencies

- pkg-config `apt install pkg-config`
- libdbus `apt install libdbus-1-dev`
- protoc `apt install protoc`
- libfontconfig `apt install libfontconfig-dev`
- libx11 `apt install libx11-dev`

## Allow USB access

1. Create udev rules entry  
   `touch /etc/udev/rules.d/69-steelseries-oled.rules`
2. Using your favourite text editor add the following line and adapt it for your device
   `SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1618", MODE="0666", GROUP="plugdev"`
3. Reload udev rules (this may not be sufficient so you may need to unplug and plug the device or restart the system)  
   `sudo udevadm control --reload-rules`

## Build from Source

1. Open Terminal
2. Download repository and go into the directory  
   `git clone TODO: URL`  
   `cd TODO: name`
3. Build  
   `cargo build --release`
4. Install  
   `cargo run --bin setup -- --dev`