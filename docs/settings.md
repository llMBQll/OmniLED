# Settings

You can fine tune behaviour of the program using the [settings file](../config/settings.lua).
All the top-level properties described below are optional and will be set to default, should any of them be missing.

## Available Settings

- [Font](#font)
- [Log Level](#log-level)
- [Keyboard](#keyboard)
- [Text Scrolling](#text-scrolling)
- [Server Port](#server-port)
- [Update Interval](#update-interval-tick-duration)

### Font

Set font used for rendering text on screen.

> _Note: While it's possible to load any font style - monospace fonts are highly recommended, especially for scrolling
text._

Property Name: `font`  
Possible Values:

- Default - loads default font [MesloLG](../steelseries_oled/assets/fonts/Meslo/).  
  `"Default"`
- Filesystem selector - loads font from specified file.
  ```lua
  font = {
    Filesystem = {
      path = ...,
      font_index = ...,
    }
  }
  ```
  `path` - Absolute path to the font file.  
  `font_index` - [Optional] font index inside the file, required only when one file defines multiple fonts.  
  Default: `0`.
- System selector - find closest match from fonts installed on your computer.
  ```lua
  font = {
    System = {
      names = ...,
      style = ...,
      weight = ...,
      stretch = ...,
    }
  }
  ```
  `names` - Array of names to search for.  
  Possible values:
    - Find by font name
      ```lua
      {
        title = ...,
      }
      ```
    - Search for a serif font  
      `"Serif"`
    - Search for a sans serif font  
      `"SansSerif"`
    - Search for a monospace font  
      `"Monospace"`
    - Search for a cursive font  
      `"Cursive"`
    - Search for a fantasy font  
      `"Fantasy"`

  `style` - [Optional] font style.  
  Possible values:
    - `"Normal"`
    - `"Italic"`
    - `"Oblique"`

  Default: `"Normal"`

  `weight`: [Optional] font weight.  
  Possible values:
    - `"Thin"`
    - `"ExtraLight"`
    - `"Light"`
    - `"Normal"`
    - `"Medium"`
    - `"SemiBold"`
    - `"Bold"`
    - `"ExtraBold"`
    - `"Black"`

  Default: `"Normal"`

  `stretch`: [Optional] font stretch.
  Possible values:
    - `"UltraCondensed"`
    - `"ExtraCondensed"`
    - `"Condensed"`
    - `"SemiCondensed"`
    - `"Normal"`
    - `"SemiExpanded"`
    - `"Expanded"`
    - `"ExtraExpanded"`
    - `"UltraExpanded"`

  Default: `"Normal"`

### Log Level

Set minimum required severity of messages to be logged.

Property Name: `log_level`  
Possible Values:

- `"Off"`
- `"Error"`
- `"Warn"`
- `"Info"`
- `"Debug"`
- `"Trace"`

Default: `"Info"`

### Keyboard

Define when to start and how often repeat key press when holding a key.  
Values are defined in [ticks](#update-interval-tick-duration).

Property Name: `keyboard_ticks_repeat_delay`  
Possible Values: non-negative 64-bit integer  
Default: `2`

Property Name: `keyboard_ticks_repeat_rate`  
Possible Values: non-negative 64-bit integer  
Default: `2`

### Text Scrolling

Define when to start and how often to shift displayed text when it doesn't fit on the screen.  
Values are defined in [ticks](#update-interval-tick-duration).

Property Name: `text_ticks_scroll_delay`  
Possible Values: non-negative 64-bit integer  
Default: `8`

Property Name: `text_ticks_repeat_delay`  
Possible Values: non-negative 64-bit integer  
Default: `2`

### Server Port

Select on which port the server will receive events from applications. When setting port `0`, OS will select any
availabe port. For any other value, server will try to bind to the specified port and exit the application if it's not
available.

Property Name: `server_port`  
Possible Values: non-negative 16-bit integer  
Default: `0`

### Update interval (Tick Duration)

This setting will define how ofter the server will process events and render updates on the screen. Lower interval will
increase responsivenes at the cost of the CPU usage.  
Update interval (or tick duration) is defined in milliseconds.

_Note: This setting will also affect all properies dependent on tick duration._

Property Name: `update_interval`  
Possible Values: non-negative 64-bit integer  
Default: `100`
