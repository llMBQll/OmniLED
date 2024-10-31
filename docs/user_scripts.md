# User Scripts

This is the main point of customization. With user scripts you can decide what, where and when to display. This can be
achieved by a combination of built-in data types, using predicates and subscribing to events.

## Widgets

Below data types are building blocks for displaying data on screen. Each of them have a few common attributes

- `position`: Point. Upper-left corner coordinates on screen.
- `size`: Size. Width and height of the object.
- `modifiers`: Modifiers. Display modifiers.

Text

- `text`: string. Text to be displayed on screen.

By default, text will be truncated to fit the size of the Text widget. If the text is longer you can either make the
widget larger or enable the `scrolling` modifier to enable automatic scrolling of text.

By default, font size will be equal to widget height. It can be changed via `font_size` modifier.

Image

- `image`: OledImage.

Bar

- `value`: float. Percentage of bar that should be filled.

[//]: # (TODO update the Bar to accept range, while using \(0, 100\) by default)
mnjhh