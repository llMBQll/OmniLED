# Devices

Any device you want to display on has to be added to config [devices.lua](../config/devices.lua) first.
At startup, only syntactic correctness of the config as well as duplicate names will be checked,
but no devices are loaded until they are registered in [scripts.lua](../config/scripts.lua).

_Note that as long as it is unique, device name does not matter, though it is easier to understand if it matches the
real device name_

## Use Already Tested Devices

Config file comes with some devices predefined, so if you see your device name you are good to go.

## Add New Devices

If your device is not on the list, you can extend it to accommodate your needs by adding an appropriate config call.  
In the following entries `Steelseries Apex 7 TKL` will be used as an example.

### Add New Steelseries Engine Device (Windows Only)

This will send all updates to Steelseries Engine (SSE) which in turn will send it to your device.

> This method should work well for a single active device, but having multiple devices loaded in this way is not tested.

Add `steelseries_engine_device` entry with the following parameters:
Required:

- `name` - unique name that will identify your device when registering it for events
- `screen_size` - width and height of the screen in pixels

**Example**

```lua
steelseries_engine_device {
    name = 'Steelseries Apex 7 TKL',
    screen_size = {
        width = 128,
        height = 40,
    },
}
```

### Add New USB Device

This method will send data directly to the USB interface responsible for handling devices' screen.
Advantage of this approach instead of just relying on SSE to do the work is less CPU usage.

Add `usb_device` entry with the following parameters:  
Required:

- `name` - unique name that will identify your device when registering it for events
- `screen_size` - width and height of the screen in pixels
- `usb_settings` - tell where data shall be sent - you can follow [this](#find-usb-settings-for-your-device) section to
  find your values
    - `vendor_id` and `product_id` - USB id of your device
    - `interface`, `endpoint`, `request_type`, `request`, `value` and `index` - USB settings that specify to which
      interface data will be written
- `memory_representation` - tell how the data should be rendered
    - `BytePerPixel` - information will be encoded in separate byte for each pixel
    - `BitPerPixel` - this will pack information about 8 pixels into each byte, and will add padding bits in the last
      byte of each row if its length is not a multiple of 8
  > Note: Data is stored as bytes of consecutive rows

Optional:

- `transform` - if the default output is not quite what you require you can transform the data into the desired format.

This is demonstrated for the `Steelseries Apex 7 TKL` keyboard which expects `BitPerPixel` layout with `0x61` byte at
the beginning and `0x00` at the end of the buffer.

**Example**

```lua
usb_device {
    name = 'Steelseries Apex 7 TKL',
    screen_size = {
        width = 128,
        height = 40,
    },
    usb_settings = {
        vendor_id = '0x1038',
        product_id = '0x1618',
        interface = '0x01',
        endpoint = '0x00',
        request_type = '0x21',
        request = '0x09',
        value = '0x0300',
        index = '0x01',
    },
    transform = function(buffer)
        local bytes = buffer:bytes()
        table.insert(bytes, 1, 0x61)
        table.insert(bytes, 0x00)
        return bytes
    end,
    memory_representation = 'BitPerPixel',
}
```

### Add New Simulator

To quickly prototype new layouts you can set up a simulator which will open a window on your screen.

Add `simulator` entry with the following parameters:  
Required:

- `name` - unique name that will identify your device when registering it for events
- `screen_size` - width and height of the screen in pixels

**Example**

```lua
simulator {
    name = 'Steelseries Apex 7 TKL',
    screen_size = {
        width = 128,
        height = 40,
    },
}
```

## Find USB Settings For Your Device

TODO