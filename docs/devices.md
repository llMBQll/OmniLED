# Devices

Before any device is used it has to be added to a config file - `devices.lua`. This file will be
read on startup and will internally register configs for the listed devices,
**but no devices will be activated at that stage**. To know how to activate a device from the
config and send data to it, read the [user scripts](user_scripts.md) page.

_Note: Device names are not actually checked to match the real device name. The only requirement is
that every name in the config is unique._

## Add device config

OmniLED supports 3 ways to display data on the screen. Each one just requires registering the
device in `devices.lua` file. Regardless of the chosen configuration type, the rendered data will
look exactly the same and only the way the data is delivered to the screen will differ.

### USB Device (Recommended)

Register any USB device using [`usb_device`](scripting_reference.md#usb_device). This is the most
flexible approach as it should work for any device. Rendered data can also be transformed via a
script to match the format expected by the device.

> Example `devices.lua` file:
> ```lua
> usb_device {
>     name = 'SteelSeries Apex 7 TKL',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
>     usb_settings = {
>         vendor_id = '0x1038',
>         product_id = '0x1618',
>         interface = '0x01',
>         endpoint = '0x00',
>         request_type = '0x21',
>         request = '0x09',
>         value = '0x0300',
>         index = '0x01',
>     },
>     transform = function(buffer)
>         local bytes = buffer:bytes()
>         table.insert(bytes, 1, 0x61)
>         table.insert(bytes, 0x00)
>         return bytes
>     end,
>     memory_representation = 'BitPerPixel',
> }
> ```
> In the above example a single usb device config was added for "SteelSeries Apex 7 TKL".
>
> It is necessary to provide all usb settings, so the device can be found, and the data is sent to
> the correct endpoint.  
> This device also expects data represent 8 bits with a single byte, thus `memory_representation`
> is set to `BitPerPixel`.  
> Additionally, the final rendered byte array is prefixed with byte `0x61` and suffixed with byte
> `0x00` to match the device's usb protocol.

### SteelSeries Devices (Windows Only)

> _Note: This approach requires SteelSeries GG software to be installed and running._

Register a SteelSeries device using 
[`steelseries_engine_device`](scripting_reference.md#steelseries_engine_device) function. This
option is the easiest as it only requires knowing the device's screen size, and SteelSeries Engine
will take the rendered data and send it to the device.

> Example `devices.lua` file:
> ```lua
> steelseries_engine_device {
>     name = 'SteelSeries Apex 7 TKL',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
> }
> ```
> In the above example a single SteelSeries device config was added for "SteelSeries Apex 7 TKL".
>
> Compared to the USB device example it's quite a bit simpler as it does not require knowing the
> usb configuration nor knowing the usb data protocol.

> Note: I was only able to test SteelSeries Engine with a single device. Handing multiple devices
> via SSE may turn out to be broken.

### Emulator

Register an emulator using [`emulator`](scripting_reference.md#emulator) function. This will not
send data to any physical device, rather it will create a new window on your desktop and show the
rendered data there. This is particularly useful for prototyping, when your device is currently not
available, or you just want to test on a bigger screen.

> Example `devices.lua` file:
> ```lua
> emulator {
>     name = 'SteelSeries Apex 7 TKL',
>     screen_size = {
>         width = 128,
>         height = 40,
>     },
> }
> ```
> In the above example a single emulator config was added for "SteelSeries Apex 7 TKL".

## Find USB Settings For Your Device

TODO