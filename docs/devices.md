# Devices

Before any device is used it has to be added to a config file - `devices.lua`. This file will be
read on startup and will internally register configs for the listed devices,
**but no devices will be activated at that stage**. To know how to activate a device from the
config and send data to it, read the [user scripts](user_scripts.md) page.

_Note: Device names are not actually checked to match the real device name. The only requirement is
that every name in the config is unique._

## Add device config

OmniLED supports multiple device registration methods. Each one just requires registering the
device in `devices.lua`. Regardless of the chosen configuration type, the rendered data will
look exactly the same and only the way the data is delivered to the screen will differ.

### HID USB Device (Recommended)

Register an HID USB device using [`hid_device`](scripting_reference.md#hid_device). This is the most
flexible approach for many USB devices and is usually the simplest path for SteelSeries-style
devices.

> Example `devices.lua` file:
>
> ```lua
> hid_device {
>     name = 'SteelSeries Apex 7 TKL',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
>     hid_settings = {
>         vendor_id = '0x1038',
>         product_id = '0x1618',
>         interface = '0x01',
>     },
>     transform = transform_data({ prepend = {0x61}, append = {0x00} }),
>     memory_layout = MemoryLayout.SteelSeries,
> }
> ```
>
> In the above example a single HID device config was added for "SteelSeries Apex 7 TKL".
>
> It is necessary to provide the HID settings so the device can be found and opened. The
> `memory_layout` field controls how the renderer formats output for the device.
>
> The optional `transform` function can be used to modify or wrap the rendered bytes before they
> are sent to the device. `transform_data` is a helper that creates a function which appends or
> prepends bytes for you. This will vary from device to device and requires having the device
> documentation or reverse engineering the device's USB protocol.

### Raw USB Device (Advanced)

Register a raw USB device using [`raw_usb_device`](scripting_reference.md#raw_usb_device) when you
need control transfer settings such as `alternate_setting`, `request_type`, `request`, `value`, and
`index`.

> Example `devices.lua` file:
>
> ```lua
> raw_usb_device {
>     name = 'Custom USB Device',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
>     usb_settings = {
>         vendor_id = '0x1038',
>         product_id = '0x1618',
>         interface = '0x01',
>         alternate_setting = '0x00',
>         request_type = '0x21',
>         request = '0x09',
>         value = '0x0300',
>         index = '0x01',
>     },
>     transform = transform_data({ prepend = {0x61}, append = {0x00} }),
>     memory_layout = MemoryLayout.SteelSeries,
> }
> ```
>
> This config is useful for devices that require raw USB control transfers instead of HID feature
> reports.

### SteelSeries Devices (Windows Only)

> _Note: This approach requires SteelSeries GG software to be installed and running._

Register a SteelSeries device using
[`steelseries_engine_device`](scripting_reference.md#steelseries_engine_device) function. This
option is the easiest as it only requires knowing the device's screen size, and SteelSeries Engine
will take the rendered data and send it to the device.

> Example `devices.lua` file:
>
> ```lua
> steelseries_engine_device {
>     name = 'SteelSeries Apex 7 TKL',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
> }
> ```
>
> In the above example a single SteelSeries device config was added for "SteelSeries Apex 7 TKL".
>
> Compared to the USB device examples it's simpler because it does not require knowing the USB
> configuration or device-specific protocol.
>
> Note: I was only able to test SteelSeries Engine with a single device. Handing multiple devices
> via SSE may turn out to be broken.

### Emulator

Register an emulator using [`emulator`](scripting_reference.md#emulator) function. This will not
send data to any physical device, rather it will create a new window on your desktop and show the
rendered data there. This is particularly useful for prototyping, when your device is currently not
available, or you just want to test on a bigger screen.

> Example `devices.lua` file:
>
> ```lua
> emulator {
>     name = 'Emulator',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
> }
> ```
>
> In the above example a single emulator config was added for "Emulator".
