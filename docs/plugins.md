# Plugins

Plugins are the data providers. OmniLED then collects that data and uses it to draw on
the screen. There are some [built-in plugins](#built-in-plugins) provided, but it's
possible to load any [custom plugin](#custom-plugins) that provides the data you need.

## Built-in Plugins

OmniLED comes with some plugins pre-installed. Each one comes with its own README file that
describes its usage and purpose.

- [audio](../omni-led-plugins/audio/README.md)
- [clock](../omni-led-plugins/clock/README.md)
- [images](../omni-led-plugins/images/README.md)
- [media](../omni-led-plugins/media/README.md)
- [system](../omni-led-plugins/system/README.md)
- [weather](../omni-led-plugins/weather/README.md)

## Loading Plugins

Plugins are loaded by registering them inside `plugins.lua` config file. This tells OmniLED where
the plugin is located and what arguments to pass to it.

To load a plugin use the global [`load_plugin`](scripting_reference.md#load_plugin) function. This
allows the script to set command line arguments.

> Example `plugins.lua` file:
>
> ``` lua
> -- load plugin located in default plugin directory
> load_plugin {
>   path = get_default_plugin_path('my_plugin'),
>   args = {
>     '--my-arg', 'MyArgValue',
>   }
> }
>
> -- load plugin located in user defined directory
> local path = 'some'
>              .. PLATFORM.PathSeparator
>              .. 'path'
>              .. PLATFORM.PathSeparator
>              .. PLATFORM.DLL_PREFIX
>              .. 'my_other_plugin'
>              .. PLATFORM.DLL_SUFFIX
> load_plugin {
>   path = path,
>   args = {
>     '--my-other-arg1', 'MyArgValue1'
>     '--my-other-arg2', 'MyArgValue2'
>   }
> }
> ```
>
> In the above example there are two plugins loaded: `my_plugin` and `my_other_plugin`.
> One uses [`get_default_plugin_path`](scripting_reference.md#get_default_plugin_path) to get the
> default path, while the other constructs a custom path manually using the
> [`PLATFORM`](scripting_reference.md#platform) constants.
> Each plugin receives different command line arguments so they get exactly what they expect.
>
> For built-in plugins' arguments refer to this [paragraph](#built-in-plugins).

## Custom Plugins

Custom plugins may be written in any language as long as they can export an C ABI interface
that implements [OmniLED's C API](../omni-led-api/omni_led_api.h).

The entry point `omni_led_run` will be called by OmniLED and will keep the plugin loaded until
this function exits.
Currently the `OmniLedApi` function table provides two functions:

- `event`: accepts an array of [CBOR-encoded](https://cbor.io/) binary data (see [CBOR types](#cbor-types))
- `log`: accepts a log level, log source location, and log message that will be then logged by OmniLED

## CBOR Types

OmniLED expects the events to be encoded as a CBOR map that can have any number of children fields.
Currently supported types are CBOR base types - booleans, numbers, strings, byte data, arrays, and maps.
It also accepts a custom type - Image.

Image (see tag number in [omni_led_api.h](../omni-led-api/omni_led_api.h))

- `format`: `string` - image data format
- `bytes`: `byte[]` - image data encoded in the provided `format`

Supported image formats:

- "Png"
- "Jpeg"
- "Gif"
- "WebP"
- "Pnm"
- "Tiff"
- "Tga"
- "Dds"
- "Bmp"
- "Ico"
- "Hdr"
- "OpenExr"
- "Farbfeld"
- "Avif"
- "Qoi"
