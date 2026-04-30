# Scripting Reference

## Constants

> ### `PLATFORM`
>
> Type: `table`
>
> This table contains platform specific constants useful for constructing paths that work
> regardless of the operating system that OmniLED is currently running on.
>
> > `ApplicationsDir: string`
> >
> > Default applications directory path.
>
> > `ConfigDir: string`
> >
> > Configuration directory path.
>
> > `ExeExtension: string`
> >
> > Extension used for executable files on current platform
> >
> > Values:
> >
> > - `"exe"` on Windows
> > - `""` (empty string) on Linux
>
> > `ExeSuffix: string`
> >
> > Suffix used for executable files on current platform
> >
> > Values:
> >
> > - `".exe"` on Windows
> > - `""` (empty string) on Linux
>
> > `PathSeparator: string`
> >
> > Separator of path components on current platform
> >
> > Values:
> >
> > - `"\"` on Windows
> > - `"/"` on Linux
>
> > `Os: string`
> >
> > Operating system name
> >
> > Values:
> >
> > - `"windows"` on Windows
> > - `"linux"` on Linux
>
> > `RootDir: string`
> >
> > Root directory path

---

> ### `PREDICATE`
>
> Type: `table`
>
> Provides predicates that can be used when registering user scripts.
>
> > `Always: fn() -> bool`
> >
> > Predicate that is always true.
>
> > `Never: fn() -> bool`
> >
> > Predicate that is always false.
>
> > `Times: fn(n: integer) -> fn() -> bool`
> >
> > Creates a predicate that will be true `n` times, after which it will always be false.

---

> ### `SCREEN`
>
> Type: `table`
>
> Provides the current screen size.
>
> > `Height: integer`
> >
> > Screen height in pixels.
>
> > `Width: integer`
> >
> > Screen width in pixels.

---

> ### `SERVER`
>
> Type: `table`
>
> This table provides the server address and start timestamp.
>
> > `Address: string`
> >
> > OmniLED server address in the format `"Ip:Port"`
>
> > `Ip: string`
> >
> > OmniLED server ip. This always is the localhost ip.
>
> > `Port: integer`
> >
> > OmniLED server port.
>
> > `Timestamp: integer`
> >
> > Unix timestamp of the server start.

## Enums

Given the following enum:

> `MyEnum`
> > VariantA
>
> > VariantB(value: integer)
>
> The variants can created like so
>
> ```lua
> local variant_a = MyEnum.VariantA
> local variant_b = MyEnum.VariantB(7)
> ```

Enum variants marked with _implicit contruct_ can be used to implicitly construct an enum if it's a function argument:

> Given:
>
> > `MyEnum`
> > > VariantA(string)
> >
> > > VariantB(string) _implicit construct_
> >
> > > VariantB(integer) _implicit construct_
>
> and a function
>
> > `fn some_fn(my_enum: MyEnum)`
>
> These are all valid:
>
> ```lua
> -- constructs MyEnum.VariantA:
> some_fn(MyEnum.VariantA('str'))
>
> -- constructs MyEnum.VariantB:
> some_fn(MyEnum.VariantB('str'))
> some_fn('str')
>
> -- constructs MyEnum.VariantC:
> some_fn(MyEnum.VariantC(1))
> some_fn(1)
> ```

---

> ### `EventKey`
>
> Used to match events in event handlers
>
> > `Regex(Regex)` _implicit construct_
> >
> > Match using regex
> >
> > `String(string)` _implicit construct_
>
> > Match the exact string

---

> ### `FontName`
>
> Font name to search for.
>
> > `Cursive`
> >
> > Search for the best matching cursive font.
> >
> > `Fantasy`
>
> > Search for the best matching fantasy font.
> >
> > `Monospace`
> >
> > Search for the best matching monospace font.
>
> > `SansSerif`
> >
> > Search for the best matching sans serif font.
>
> > `Serif`
> >
> > Search for the best matching serif font.
>
> > `Title(title: string)`
> >
> > Search for a font with a specific `title`.

---

> ### `FontSelector`
>
> Different strategies for selecting fonts.
>
> > `Default`
> >
> > Load default font.
> >
> > `Filesystem(selector: FilesystemSelector)`
> >
> > Load a font using a file system path.
> >
> > `System(selector: SystemSelector)`
> >
> > Load for a system-installed font.

---

> ### `FontSize`
>
> Specifies the text font size inside of `Text` widget.
>
> > `Auto`
> >
> > Calculate font size to fit any text in the widget's height.
>
> > `AutoUpper`
> >
> > Calculate font size to fit any text that doesn't have any "descendants". Useful for text that
> > consists only of uppercase characters or numbers.
>
> > `Value(n: integer)` _implicit contruct_
> >
> > Set font size to be exactly `n`, regardless of widget size.

---

> ### `FontStretch`
>
> Font stretch, from most condensed to most stretched.
>
> > `UltraCondensed`
>
> > `ExtraCondensed`
>
> > `Condensed`
>
> > `SemiCondensed`
>
> > `Normal`
>
> > `SemiExpanded`
>
> > `Expanded`
>
> > `ExtraExpanded`
>
> > `UltraExpanded`

---

> ### `FontStyle`
>
> Font style.
>
> > `Normal`
>
> > `Italic`
>
> > `Oblique`

---

> ### `FontWeight`
>
> Font weight, from thinnest to boldest.
>
> > `Thin`
>
> > `ExtraLight`
>
> > `Light`
>
> > `Normal`
>
> > `Medium`
>
> > `SemiBold`
>
> > `Bold`
>
> > `ExtraBold`
>
> > `Black`

---

> ### `ImageFormat`
>
> Image format.
>
> > `Avif`
>
> > `Bmp`
>
> > `Dds`
>
> > `Farbfeld`
>
> > `Gif`
>
> > `Hdr`
>
> > `Ico`
>
> > `Jpeg`
>
> > `OpenExr`
>
> > `Pcx`
>
> > `Png`
>
> > `Pnm`
>
> > `Qoi`
>
> > `Tga`
>
> > `Tiff`
>
> > `WebP`

---

> ### `LogLevel`
>
> Log level filter, selecting one value will also activate all values above it, e.g. enabling
> `Info`, will also enable `Warn` and `Error`.
>
> > `Off`
> >
> > Disables all logging.
>
> > `Error`
> >
> > Allow logging errors and above.
>
> > `Warn`
> >
> > Allow logging warnings and above.
>
> > `Info`
> >
> > Allow info logging and above.
>
> > `Debug`
> >
> > Allow debug logging and above.
>
> > `Trace`
> >
> > Allow tracing and above.

---

> ### `MemoryLayout`
>
> Memory layout strategy for data sent to devices via USB.
>
> > `BytePerPixel` | `SteelSeries`
> >
> > Represent information about each pixel in a separate byte.
>
> > `BitPerPixel`
> >
> > Pack information about 8 pixels into each byte in a way byte represents 8 consecutive pixels in
> > a row. This will also add padding bits at the end of each row if the row length is not a
> > multiple of 8.
>
> > `BitPerPixelVertical` | `SteelSeries2`
> >
> > Pack information about 8 pixels into each byte in a way byte represents 8 consecutive pixels in
> > a column. This will also add padding bits at the end of each column if the column length is not
> > a multiple of 8.

---

> ### `Repeat`
>
> Repeat strategy for a widget. Applies to scrolling text and animated images.
>
> > `Once`
> >
> > Repeats the script until the animation is finished, even if it takes longer than the duration
> > specified for layout. This way the entire animation is displayed exactly once.
>
> > `ForDuation`
> >
> > Repeats the script for the time of its duration. This will run the animation for an exact
> > duration,
> > but can cut off mid-animation if the time runs out.

## Functions

> ### `dump`
>
> Type: `fn(value: any) -> string`
>
> Pretty print any lua value into a string.

---

> ### `emulator`
>
> Type: `fn(settings: EmulatorSettings)`
>
> Registers emulator with a given configuration.

---

> ### `get_default_path`
>
> Type: `fn(name: string) -> string`
>
> Return the full, system-specific path for a given application name. This assumes the application
> is in the default installation directory.
>
> Examples:
>
> - `C:\Users\<USERNAME>\AppData\Roaming\OmniLED\bin\<NAME>.exe` on Windows
> - `/home/<USERNAME>/.config/OmniLED/bin/<NAME>` on Linux

---

> ### `hid_device`
>
> Type: `fn(settings: HidDeviceSettings)`
>
> Registers HID USB device with a given configuration.

---

> ### `load_app`
>
> Type: `fn(config: Config)`
>
> Starts an application with the given configuration.

---

> ### `marked_table`
>
> Type: `fn(table: table) -> table`
>
> Sets a metatable that marks this table as a key-value table.
> Returns the passed table for convenience.

---

> ### `raw_usb_device`
>
> Type: `fn(settings: RawUsbDeviceSettings)`
>
> Registers raw USB device with a given configuration.

---

> ### `steelseries_engine_device`
>
> Type: `fn(settings: SteelSeriesEngineDeviceSettings)`
>
> Registers SteelSeries device with a given configuration.

---

> ### `transform_data`
>
> Type: `fn(extra: ExtraBytes) -> (fn(buffer: Buffer) -> [byte])`
>
> Creates a tranform function that will add extra bytes to the rendered buffer.

## Objects

> ### `Buffer`
>
> Byte buffer that contains the rendered data.
>
> > `bytes`: `fn(self) -> [byte]`
> >
> > Get a flat array of bytes in a device-specific memory layout.

---

> ### `Regex`
>
> Used to check if a string matches a regex pattern
>
> > `new`: `fn(pattern: string) -> Regex`
> >
> > Creates a new regex object with a given pattern.
> >
> > This will return an error if pattern fails to compile.
>
> > `matches`: `fn(self, string: string) -> bool`
> >
> > Checks if `string` matches the regex pattern

---

> ### `EVENTS`
>
> Register callbacks for specific events. This, combined with script predicates, is useful when the
> screen builder with default screen management doesn't quite cut it.
>
> > `register: fn(self, key: EventKey, callback: fn(event: string, value: any)) -> EventHandle`
> >
> > Register a callback for any event. When the callback is triggered, the event name and its value
> > will be passed as callback arguments.  
> > Returns an EventHandle object that can be used to unsubscribe from the event.
>
> > `unregister: fn(self, handle: EventHandle)`
> >
> > Unregister from an event using a handle received when registering.
>
> > `send: fn(self, event: string, value: any)`
> >
> > Send any event from user scripts. Due to lua limitations, for
> > [recursive updates](user_scripts.md#application_update_events) each table must be marked using `marked_table`
> > function.
> >
> > `event` must only consist of uppercase aplhanumeric characters and cannot start with a number.
> >
> > Example
> >
> > ```lua
> > EVENTS:send('MY_EVENT', marked_table {
> >   Data = 1,
> >   Nested = marked_table {
> >     MoreData = 2,
> >   },
> > }
> > ```

---

> ### `LOG`
>
> Type: `table`
>
> Provides logging functions for scripts.
>
> > `debug: fn(self, message: string)`
> >
> > Log a debug message
>
> > `error: fn(self, message: string)`
> >
> > Log an error message
>
> > `info: fn(self, message: string)`
> >
> > Log an info message
>
> > `trace: fn(self, message: string)`
> >
> > Log a trace message
>
> > `warn: fn(self, message: string)`
> >
> > Log a warning message

---

> ### `SCREEN_BUILDER`
>
> Screen builder allows to put together layouts and screen setups for devices. This has to be
> repeated for every device individually.
>
> > `new: fn(self, name: string)`
> >
> > Begin a builder to register screen layouts for a given device. `name` must be a device with an
> > existing config entry.
>
> > `register: fn(self)`
> >
> > Finalize the builder and register all provided scripts and shortcuts for the provided device.
> > Without this call, no scripts will be registered.
>
> > `with_layout: fn(self, layout: Layout)`
> >
> > Add a single layout. Useful for more advanced custom configuration.
> >
> > _Not compatible with `with_layout_group` and `with_layout_group_toggle`._
>
> > `with_layout_group: fn(self, layouts: [Layout])`
> >
> > Add a screen with an array of user scripts, sorted in order of decreasing priority - first
> > entry has the highest priority. This function can be called multiple times to register many
> > screens for a single device.
> >
> > _Not compatible with `with_layout`._
>
> > `with_layout_group_toggle: fn(self, shortcut: [string])`
> >
> > Set a shortcut to toggle between screens. It will go sequentially through each screen, and wrap
> > around to the first at the end. Required if there is more then one screen being registered.
> >
> > _Not compatible with `with_layout`._

---

> ### `SHORTCUTS`
>
> Register shortcuts to perform custom actions. This provides a bit of convenience over just using
> the [`EVENTS`](#events) allowing for listening for a complete combination of keys as well as
> allowing customizable for auto-repeating behaviour on hold.
>
> _See [keyboard settings](settings.md#keyboard)_
>
> > `register: fn(self, keys: [string], action: fn())`
> >
> > Register a key combination and an action that will be executed when the combination is pressed.

## Types

> ### `Config`
>
> Configuration for starting an application.
>
> > `path: string`
> >
> > Path to the executable.
>
> > `args: [string]`
> >
> > _Optional_. Default: `[]`.
> >
> > Command line arguments to pass to the executable.

---

> ### `EmulatorConfig`
>
> Configuration for a device emulator.
>
> > `name: string`
> >
> > Unique name that identifies this configuration.
>
> > `screen_size: Size`
> >
> > Screen size to use for emulator display.

---

> ### `ExtraBytes`
>
> Specify where and what bytes should be added to rendered buffer.
>
> > `prepend: [u8]`
> >
> > _Optional_. Default: `[]`.
> >
> > Prepend bytes to rendered buffer.
>
> > `append: [u8]`
> >
> > _Optional_. Default: `[]`.
> >
> > Append bytes to rendered buffer.

---

> ### `FilesystemSelector`
>
> Load a font using a file system path.
>
> > `path: string`
> >
> > Full path pointing to the font file.
>
> > `font_index: integer`
> >
> > _Optional_. Default: `0`.
> >
> > If the file contains multiple faces, you may need to provide the index to select the proper one.

---

> ### `ImageData`
>
> Contains image bytes and format
>
> > `format`: `ImageFormat`
> >
> > Image format, required to properly load image bytes.
>
> > `bytes`: `[byte]`
> >
> > Image bytes stored in format specified by the `format` property.

---

> ### `Layout`
>
> Represents a user-defined script that runs on specific events and creates a layout to be rendered.
>
> > `layout_fn: fn() -> LayoutData`
> >
> > Function that will return a valid [`LayoutData`](#layoutdata) when triggered.
>
> > `run_on: [string]`
> >
> > List of events that can trigger the script.
>
> > `predicate: fn() -> bool`
> >
> > _Optional_. Default: `PREDICATE.Always`.
> >
> > Additional predicate to assert if a script should be run. If it returns `false`, the user
> > script will not be run despite receiving an event specified in the `run_on` array.

---

> ### `LayoutData`
>
> Specify the widget layout and other properties required for rendering on screen.
>
> > `widgets: [Widget]`
> >
> > Array of widgets that compose the layout. Widgets are rendered in the order they are in the
> > array.
>
> > `duration: integer`
> >
> > How many milliseconds can the layout be shown on the screen before it's allowed to be
> > overridden. Higher priority layouts can always override lower priority, regardless of the
> > remaining duration.

---

> ### `Modifiers`
>
> Represents display options for widgets.
>
> > `clear_background`: `bool`
> >
> > _Optional_. Default: `false`.
> >
> > Resets all pixels in the widget's area before drawing the widget's content.
>
> > `flip_horizontal`: `bool`
> >
> > _Optional_. Default: `false`.
> >
> > Flips the content horizontally along the middle of the widget’s width.
>
> > `flip_vertical`: `bool`
> >
> > _Optional_. Default: `false`.
> >
> > Flips the content vertically along the middle of the widget’s height.
>
> > `negative`: `bool`
> >
> > _Optional_. Default: `false`.
> >
> > Swaps on and off pixels for a given widget.

---

> ### `Point`
>
> Represents a coordinate in a 2D space with an origin `(0, 0)` in the top left corner of the
> screen.
>
> > `x`: `integer`
> >
> > X-coordinate.
> >
>
> > `y`: `integer`
> >
> > Y-coordinate.

---

> ### `Range`
>
> Represents a numeric range.
>
> > `min`: `float`
> >
> > Lower end of the range (inclusive).
>
> > `max`: `float`
> >
> > Upper end of the range (inclusive).

---

> ### `Size`
>
> Represents object size.
>
> > `height`: `integer`
> >
> > Height in pixels.
>
> > `width`: `integer`
> >
> > Width in pixels.

---

> ### `SteelSeriesEngineDeviceSettings`
>
> Configuration for a device managed via SteelSeriesEngine.
>
> > `name: string`
> >
> > Unique name that identifies this configuration.
>
> > `screen_size: Size`
> >
> > Screen size of the SteelSeries device display.

---

> ### `SystemSelector`
>
> > `names: [FontName]`
> >
> > List of font names in decreasing priority.
>
> > `style: FontStyle`
> >
> > _Optional_. Default: `"Normal"`.
> >
> > Font style to search for.
>
> > `weight: FontWeight`
> >
> > _Optional_. Default: `"Normal"`.
> >
> > Font weight to search for.
>
> > `stretch: FontStretch`
> >
> > _Optional_. Default: `"Normal"`.
> >
> > Font stretch to search for.

---

> ### `HidDeviceSettings`
>
> Configuration for an HID USB device.
>
> > `name: string`
> >
> > Unique name that identifies this configuration.
>
> > `screen_size: Size`
> >
> > Screen size of the HID device display.
>
> > `memory_layout: MemoryLayout`
> >
> > Memory layout of the renderer output.
>
> > `transform: fn(buffer: Buffer) -> [byte]`
> >
> > _Optional_. Default: No transformation of rendered data.
> >
> > Function that will transform rendered `buffer` into the final representation expected by the
> > device. Data inside `buffer` is in a format specified by `memory_layout` field.
>
> > `hid_settings: HidSettings`
> >
> > HID-specific settings to identify and configure the USB device.

---

> ### `HidSettings`
>
> HID USB device settings.
>
> > `vendor_id: string`
> >
> > Device vendor ID.
>
> > `product_id: string`
> >
> > Device product ID.
>
> > `interface: string`
> >
> > USB interface number.

---

> ### `RawUsbDeviceSettings`
>
> Configuration for a raw USB device.
>
> > `name: string`
> >
> > Unique name that identifies this configuration.
>
> > `screen_size: Size`
> >
> > Screen size of the USB device display.
>
> > `memory_layout: MemoryLayout`
> >
> > Memory layout of the renderer output.
>
> > `transform: fn(buffer: Buffer) -> [byte]`
> >
> > _Optional_. Default: No transformation of rendered data.
> >
> > Function that will transform rendered `buffer` into the final representation expected by the
> > device. Data inside `buffer` is in a format specified by `memory_layout` field.
>
> > `usb_settings: RawUsbSettings`
> >
> > Raw USB settings to identify and configure the USB device.

---

> ### `RawUsbSettings`
>
> Raw USB device settings.
>
> > `vendor_id: string`
> >
> > Device vendor ID.
>
> > `product_id: string`
> >
> > Device product ID.
>
> > `interface: string`
> >
> > USB interface number.
>
> > `alternate_setting: string`
> >
> > Alternate setting.
>
> > `request_type: string`
> >
> > USB control transfer request type.
>
> > `request: string`
> >
> > USB control transfer request ID.
>
> > `value: string`
> >
> > USB control transfer value.
>
> > `index: string`
> >
> > USB control transfer index.

## Widgets

Widgets are the building blocks for displaying data on screen. Combine them to create full layouts.

All widgets have the following common attributes in addition to widget-specific ones.

> ### Common attributes
> >
> > `position`: `Point`
> >
> > Position of the upper-left corner of the widget.
>
> > `size`: `Size`
> >
> > Widget size.
>
> > `modifiers`: `Modifiers`
> >
> > _Optional_. Default: No modifiers.
> >
> > Display modifiers for the widget.

> ### `Bar`
>
> A widget that represents a progress or status bar.
>
> > `range`: `Range`
> >
> > _Optional_. Default: `{0.0, 100.0}`
> >
> > The minimum and maximum values that can be displayed on the bar.
>
> > `value`: `float`
> >
> > Amount of the bar that will be filled depends on where `value` lies in the `range`.  
> > It is calculated as: `(value - range.min) / (range.max - range.min) * 100%`.
>
> > `vertical`: `bool`
> >
> > _Optional_. Default: `false`
> >
> > Specifies if the bar should render from bottom to top instead of left to right.

---

> ### `Image`
>
> A widget that displays an image.
>
> > `image`: `ImageData`
> >
> > The image data to display on the screen.  
> > This image will be scaled from its original size to the dimensions of the widget.
>
> > `animated`: `bool`
> >
> > _Optional_. Default: `false`
> >
> > Specifies if the image should be animated. Unless set to `true`, event with supported image
> > formats, only a static image will be rendered.
>
> > `animation_group`: `integer`
> >
> > _Optional_. Default: `0`
> >
> > Sets the animation group for the widget. All animations within a single animation group are
> > synced,
> > except for the default group `0`, where all animations are independent.
>
> > `animation_ticks_delay`: `integer`
> >
> > _Optional_. Default: No value
> >
> > Overrides [global animation setting](settings.md#animation) for this widget. Applies only for
> > `animated` images.
> >
> > **Changing this value after initially setting it for a given widget is undefined behaviour.**
>
> > `animation_ticks_rate`: `integer`
> >
> > _Optional_. Default: No value
> >
> > Overrides [global animation setting](settings.md#animation) for this widget. Applies only for
> > `animated` images.
> >
> > **Changing this value after initially setting it for a given widget is undefined behaviour.**
>
> > `repeats: Repeat`
> >
> > _Optional_. Default: `ForDuration`.
> >
> > Specifies the repeat strategy, applies only for animated images.
>
> > `threshold`: `integer`
> >
> > _Optional_. Default: `128`
> >
> > Specifies the threshold from range `[0, 255]` used to convert image to a black and white image.
> > Light values below the threshold will be converted to black, and values above the threshold will
> > be converted to white.

---

> ### `Text`
>
> A widget that displays text.
>
> > `text`: `string`
> >
> > Text to display on the screen.
>
> > `scrolling`: `bool`
> >
> > _Optional_. Default: `false`
> >
> > Specifies if the text should scroll if it is too long to fit within the widget's width.
>
> > `animation_group`: `integer`
> >
> > _Optional_. Default: `0`
> >
> > Sets the animation group for the widget. All animations within a single animation group are
> > synced,
> > except for the default group `0`, where all animations are independent.
>
> > `animation_ticks_delay`: `integer`
> >
> > _Optional_. Default: No value
> >
> > Overrides [global animation setting](settings.md#animation) for this widget. Applies only for
> > `scrolling` text.
> >
> > **Changing this value after initially setting it for a given widget is undefined behaviour.**
>
> > `animation_ticks_rate`: `integer`
> >
> > _Optional_. Default: No value
> >
> > Overrides [global animation setting](settings.md#animation) for this widget. Applies only for
> > `scrolling` text.
> >
> > **Changing this value after initially setting it for a given widget is undefined behaviour.**
>
> > `font_size`: `FontSize`
> >
> > _Optional_. Default: `"Auto"`.
> >
> > Sets the font size of the text.
>
> > `repeats: Repeat`
> >
> > _Optional_. Default: `ForDuration`.
> >
> > Specifies the repeat strategy, applies only for scrolling text.
>
> > `text_offset`: `integer`
> >
> > _Optional_. Default: Calculated automatically based on the `font_size`.
> >
> > Determines the offset of the text from the bottom of the widget.
