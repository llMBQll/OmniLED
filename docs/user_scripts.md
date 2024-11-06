# User Scripts

This is the main point of customization. With user scripts you can decide what, where and when to display. This can be
achieved by a combination of built-in data types, using predicates and subscribing to events.

## Widgets

Below data types are building blocks for displaying data on screen. Each of them have a few common attributes

- `position`: Point. Upper-left corner coordinates on screen.
- `size`: Size. Width and height of the object.
- `modifiers`: Modifiers. Display modifiers.

### Text

- `text`: string. Text to be displayed on screen.

By default, text will be truncated to fit the size of the Text widget. If the text is longer you can either make the
widget larger or enable the `scrolling` modifier to enable automatic scrolling of text.

By default, font size will be equal to widget height. It can be changed via `font_size` modifier.

### Image

- `image`: OledImage. Image that will be displayed on screen.

Image will be scaled from its original size to widget's dimensions.

### Bar

- `value`: float. Amount of the bar that will be filled depends on where `value` lies in the `range`. It is calculated
  using the following formula `(value - range.min) / (range.max - range.min) * 100%`.
- `range`: Range. Minimum and Maximum values that can be displayed on the bar.

`range` is optional and will be `[0.0, 100.0]` by default.

## Data Types

Point

- `x`: integer. X-coordinate
- `y`: integer. Y-coordinate

```lua
point = {
    x = 1,
    y = 2,
}
```

Range

- `min`: float. Lower end of the range (inclusive)
- `max`: float. Upper end of the range (inclusive)

```lua
range = {
    min = 10.1,
    max = 99.9,
}
```

Size

- `width`: integer. Width value
- `height`: integer. Height value

```lua
size = {
    width = 7,
    height = 8,
}
```

OledImage

- `size`: Size. Image size in pixels
- `bytes`: \[byte\]. Row-major black and white image data with one byte per pixel. All non-zero values will result in the pixels being on.

`size.width * size.height` must be equal to length of the `bytes` array.

```lua
image = {
    size = {
        width = 2,
        height = 2,
    },
    bytes = { 0, 1, 0, 1 },
}
```

Modifiers

Additional display options for widgets.

- `clear_background`: `bool` - Resets all pixels in widget's area before drawing the widget's content.
- `flip_horizontal`: `bool` – Flips the content horizontally along the middle of the widget width.
- `flip_vertical`: `bool` – Flips the content vertically along the middle of the widget height.
- `font_size`: `integer` – Sets the font size, defaulting to the widget height. Currently applies only to [Text](#text).
- `negative`: `bool` - Swap on and off pixels for a given widget.
- `scrolling`: `bool` – Enables scrolling for content that exceeds widget size. Currently applies only to [Text](#text).
- `vertical`: `bool` – Orients the widget vertically. Currently applies only to [Bar](#bar).