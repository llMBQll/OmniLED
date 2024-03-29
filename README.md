# Steelseries OLED

## Linux Dependencies

Pkg Config
sudo apt install pkg-config -y

D-Bus
sudo apt install libdbus-1-dev -y

Protobuf compiler
Download latest from protobuf #TODO determine minimal supported version

Font Config
sudo apt install libfontconfig-dev -y

X11 
sudo apt install libx11-dev -y

## Allow access to usb device

Add rules file
```
❯ cat /etc/udev/rules.d/69-steelseries-oled.rules
SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="1618", MODE="0666", GROUP="plugdev"
```

Reload rules (additionally may need to unplug and plug the device or restart system)
```
sudo udevadm control --reload-rules
```