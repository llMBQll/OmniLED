# Lua Interfaces

## Constants

> ### `PLATFORM`
>
> Type: `table`
>
> This table contains platform specific constants useful for constructing paths that work regardless of the
> operating system that OmniLED is currently running on.
>
> > `ApplicationsDir: string`
> >
> > Default applications directory path.
>
> > `ExeExtension: string`
> >
> > Extension used for executable files on current platform
> >
> > Values:
> > - `"exe"` on Windows
> > - `""` (empty string) on Linux
>
> > `ExeSuffix: string`
> >
> > Suffix used for executable files on current platform
> >
> > Values:
> > - `".exe"` on Windows
> > - `""` (empty string) on Linux
>
> > `PathSeparator: string`
> >
> > Separator of path components on current platform
> >
> > Values:
> > - `"\"` on Windows
> > - `"/"` on Linux
>
> > `Os: string`
> >
> > Separator of path components on current platform
> >
> > Values:
> > - `"windows"` on Windows
> > - `"linux"` on Linux

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

> ### `Repeat`
>
> Repeat strategy for a widget. Currently, this only applies to scrolling text.
>
> > `Once`
> >
> > Repeats the script until the text is fully scrolled, even if it takes longer than the duration specified for layout.
> > This way the entire text is displayed exactly once.
>
> > `ForDuation`
> >
> > Repeats the script for the time of its duration. This will scroll text for an exact duration, but can cut off mid
> > scrolling if the time runs out.

## Functions

> ### `load_app`
>
> Type: `fn(path: string, args: [string])`
>
> Starts an application at `path` and passes the `args` as command line arguments.

> ### `get_default_path`
>
> Type: `fn(name: string) -> string`
>
> Return the full, system-specific path for a given application name. This assumes the application is in the default
> installation directory.
>
> Examples:
> - `C:\Users\<USERNAME>\AppData\Roaming\OmniLED\bin\<NAME>.exe` on Windows
> - `/home/<USERNAME>/.config/OmniLED/bin/<NAME>` on Linux

## Objects

> ### `SCREEN_BUILDER`
>
> Screen builder allows to put together layouts and screen setups for devices. This has to be repeated for every device
> individually.
>
> > `new: fn(self, name: string)`
> >
> > Begin a builder to register screen layouts for a given device. `name` must be a device with an existing config
> > entry.
>
> > `register: fn(self)`
> >
> > Finalize the builder and register all provided scripts and shortcuts for the provided device. Without this call, no
> > scripts will be registered.
>
> > `with_screen: fn(self, user_scripts: [UserScript])`
> >
> > Add a screen with an array of user scripts, sorted in order of decreasing priority - first entry has the highest
> > priority. This function can be called multiple times to register many screens for a single device.
> >
> > _Not compatible with `with_script`._
>
> > `with_screen_toggle: fn(self, shortcut: [string])`
> >
> > Set a shortcut to toggle between screens. It will go sequentially through each screen, and wrap around to the first
> > at the end. Required if there is more then one screen being registered.
> >
> > _Not compatible with `with_script`._
>
> > `with_script: fn(self, user_script: UserScript)`
> >
> > Add a single user script. Useful for more advanced custom configuration.
> >
> > _Not compatible with `with_screen` and `with_screen_toggle`._

---

> ### `SHORTCUTS`
>
> Register shortcuts to perform custom actions. This, combined with script predicates, is useful when the screen builder
> with default screen management doesn't quite cut it.
>
> > `register: fn(self, keys: [string], action: fn())`
> >
> > Register a key combination and an action that will be executed when the combination is pressed.

## Types

> ### `Layout`
>
> Specify the widget layout and other properties required for rendering on screen.
>
> > `widgets: [Widget]`
> >
> > Array of widgets that compose the layout. Widgets are rendered in the order they are in the array.
>
> > `duration: integer`
> >
> > How many milliseconds can the layout be shown on the screen before it's allowed to be overridden. Higher priority
> > layouts can always override lower priority, regardless of the remaining duration.
>
> > `repeats: Repeat`
> >
> > _Optional_. Default: `Once`.
> >
> > Specifies the repeat strategy, which is only applicable to scrolling text for now.

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

> ### `Offset`
>
> Specifies the text offset from the bottom of a `Text` widget.
>
> > Offset is a mixture of an integer and an enum. It can take the following values:
> > - `n: integer`: Offset by `n` pixels.
> > - `"Auto"`: Calculate offset to fit any text in the widget's height.
> > - `"AutoUpper"`: Calculate offset to fit any text that doesn't have any "descendants". Useful for text that consists
      only of uppercase characters or numbers.

---

> ### `OledImage`
>
> Represents a black-and-white image.
>
> > `size`: `Size`
> >
> > Source image size. To adjust image size on screen, set the appropriate widget size to the desired value.
>
> > `bytes`: `[byte]`
> >
> > Row-major black-and-white image data with one byte per pixel. All non-zero values will result in the pixels being
> > on.
> > `size.width * size.height` must equal the length of the `bytes` array.

---

> ### `Point`
>
> Represents a coordinate in a 2D space with an origin `(0, 0)` in the top left corner of the screen.
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

> ### `UserScript`
>
> Represents a user-defined script that runs on specific events and creates a layout to be rendered.
>
> > `layout: fn() -> Layout`
> >
> > Function that will return a valid `Layout` when triggered.
>
> > `run_on: [string]`
> >
> > List of events that can trigger the script.
>
> > `predicate: fn() -> bool`
> >
> > _Optional_. Default: `PREDICATE.Always`.
> >
> > Additional predicate to assert if a script should be run. If it returns `false`, the user script will not be run
> > despite receiving an event specified in the `run_on` array.

## Widgets

Widgets are the building blocks for displaying data on screen. Combine them to create full layouts.

All widgets have the following common attributes in addition to widget-specific ones.

> ### Common attributes
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
> > `image`: `OledImage`
> >
> > The image data to display on the screen.  
> > This image will be scaled from its original size to the dimensions of the widget.

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
> > `font_size`: `integer`
> >
> > _Optional_. Default: Calculated to fit within the widget's height.
> >
> > Sets the font size of the text.
>
> > `text_offset`: `Offset`
> >
> > _Optional_. Default: `"Auto"`.
> >
> > Determines the offset of the text from the bottom of the widget.