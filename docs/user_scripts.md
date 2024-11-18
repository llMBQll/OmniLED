# User Scripts

This is the main point of customization. With user scripts you can decide what, where and when to display. This can be
achieved by a combination of built-in data types, using predicates and subscribing to events.

## Drawing on The Screen

## Screen Builder

Screen builder ties everything together, allowing to register screens with layouts, and shortcuts for specific
devices.

Global `SCREEN_BUILDER` object.

[//]: # (TODO link example)

### Methods

- `new`: `fn(name: string)` - begin a builder to register for device a given device. Device `name` must be registered in
  devices.lua

  [//]: # (TODO link devices.lua)
- `with_screen`: `fn(user_scripts: [UserScripts])` - add a screen with an array of user scripts
  (see [UserScript](#user-script)), sorted in order of decreasing priority - first entry has the highest priority.  
  This function can be called multiple times to register many screens for a single device. You will need to register a
  shortcut to switch between them using `with_screen_toggle`.
- `with_screen_toggle`: `fn(shortcut: [string])` - set a shortcut to toggle between screens. It will go sequentially
  through each screen, and wrap around to the first at the end. Only required if there is more then one screen being
  registered.
- `with_script`: `fn(user_script: UserScript)` - add a user script. Useful for custom screen management. It requires
  registering shortcuts manually (see [Shortcuts](#shortcuts)).  
  This method is mutually exclusive with `with_screen` and `with_screen_toggle`.
- `register`: `fn()` - end registering device. This function actually performs the actions set up with previous
  functions.

### Types

#### User Script

- `layout`: `fn() -> Layout` - function that returns the layout of the
- `run_on`: `[string]` - array of events that can trigger this user script.
- `predicate`: `fn() -> bool` - additional way to filter when the layout should be shown. Returning `true` allows script
  to be run, returning `false` rejects the update. Predicate is only checked if an event fired and was found in `run_on`
  array. This field is optional. There are three predefined predicates:
    - `PREDICATE.Always` - same as not providing the predicate at all, will always return `true`. Mostly meant for a
      placeholder.
    - `PREDICATE.Never` - will always return `false`. Mostly meant for a placeholder.
    - `PREDICATE.Times(n)` - will only return `true` the first `n` times, after that it will always return `false`.

#### Layout

- `widgets`: `[Widget]` - array of widgets that compose the layout. Widgets are rendered in the provided order.
- `duration`: `integer` - how long can the layout be shown on the screen before it's allowed to be overridden. Higher
  priority layouts can always override lower priority, regardless of the remaining duration though.
- `repeats`: `Repeat` - specifies the repeat strategy, which is only applicable to scrolling text for now.  
  Default value is `Once`.

#### Repeat

`Repeat` is an enum that specifies how the repeating should be handled.

- `Once` - Repeats the script until the text is fully scrolled, even if it takes longer than the duration specified for
  its layout. This way the entire text is displayed exactly once.
- `ForDuration` - Repeats the script for the time specified in `duration` field. This may allow switching to other
  screens in the middle of scrolling text.

## Shortcuts

You can register shortcuts to perform custom actions. This, combined with script predicates, is useful when the screen
builder with default screen management doesn't quite cut it.

Global `SHORTCUTS` object.

### Methods

`register`: `fn(keys: [string], action: fn())`. Register a key combination and an action that will be executed when the
combination is pressed.

[//]: # (TODO link example)

## Widgets

Below data types are building blocks for displaying data on screen. Each of them have a few common attributes

- `position`: `Point`. Upper-left corner coordinates on screen.
    - required
    - see: [Point](#point)
- `size`: `Size`. Width and height of the object.
    - required
    - see: [Size](#Size)
- `modifiers`: `Modifiers`. Display modifiers.
    - optional
    - see: [Modifiers](#Modifiers)

### Bar

- `value`: `float`. Amount of the bar that will be filled depends on where `value` lies in the `range`. It is calculated
  using the following formula `(value - range.min) / (range.max - range.min) * 100%`.
    - required
- `range`: `Range`. Minimum and Maximum values that can be displayed on the bar.
    - default: `[0.0, 100.0]`
    - see: [Range](#range)
- `vertical`: `bool`. Makes the bar from bottom to top, instead of left to right.
    - default: `false`

### Image

- `image`: `OledImage`. Image that will be displayed on screen.
    - required
    - see: [OledImage](#oledimage)

Image will be scaled from its original size to widget's dimensions.

### Text

- `text`: `string`. Text to be displayed on screen.
    - required
- `scrolling`: `bool`. Scroll text if it's too long.
    - default: `false`
- `font_size`: `integer` – Sets the font size.
    - default: Calculated to fit widget's height
- `text_offset`: `Offset` - Set text offset from the bottom of the widget.
    - default: `Auto`
    - see: [Offset](#offset)

By default, text will be truncated to fit the size of the Text widget. If the text is longer you can either make the
widget larger or enable the `scrolling` modifier to enable automatic scrolling of text.

By default, font size will be equal to widget height. It can be changed via `font_size` modifier.

## Data Types

### Point

- `x`: `integer`. X-coordinate
    - required
- `y`: `integer`. Y-coordinate
    - required

```lua
point = {
    x = 1,
    y = 2,
}
```

### Range

- `min`: `float`. Lower end of the range (inclusive)
    - required
- `max`: `float`. Upper end of the range (inclusive)
    - required

```lua
range = {
    min = 10.1,
    max = 99.9,
}
```

### Size

- `width`: `integer`. Width value
    - required
- `height`: `integer`. Height value
    - required

```lua
size = {
    width = 7,
    height = 8,
}
```

### Offset

Offset can take the following values:

- integer. Offset by number of pixels.
- `Auto`. Calculate offset to fit any text in widget's height
- `AutoUpper`. Calculate offset to fit any text that doesn't have any "descendants". Useful for uppercase text or
  numbers.

```lua
offset = 1
offset = 'Auto'
offset = 'AutoUpper'
```

### OledImage

- `size`: `Size`. Image size in pixels
    - required
    - see: [Size](#Size)
- `bytes`: `[byte]`. Row-major black and white image data with one byte per pixel. All non-zero values will result in
  the pixels being on.  
  `size.width * size.height` must be equal to length of the `bytes` array.
    - required

```lua
image = {
    size = {
        width = 2,
        height = 2,
    },
    bytes = { 0, 1, 0, 1 },
}
```

### Modifiers

Additional display options for widgets.

- `clear_background`: `bool`. Resets all pixels in widget's area before drawing the widget's content.
    - default: `false`
- `flip_horizontal`: `bool`. Flips the content horizontally along the middle of the widget width.
    - default: `false`
- `flip_vertical`: `bool`. Flips the content vertically along the middle of the widget height.
    - default: `false`
- `negative`: `bool`. Swap on and off pixels for a given widget.
    - default: `false`
