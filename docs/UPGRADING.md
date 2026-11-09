# Upgrading

> [!TIP]
> Sometimes it might be easier to backup your config to another place and delete the original. On application launch,
> OmniLED will write default configs which then you can edit with your (possibly adjusted to new API) changes.

## 0.11.\* -> 0.12.\*

### `Settings` is a table-like object now

`Settings` is a LuaUserData object shared across all scripts with all fields being accessible as if it was a regular
Lua table.

Replace the old constructor call with a table assignment:

```lua
-- 0.11
Settings {
  update_interval = Duration.from_millis(100),
}

-- 0.12 - Works same as 0.11, all missing fields are set to default values
Settings = {
  update_interval = Duration.from_millis(100),
}

-- 0.12 - Individual assignment, only mentioned field is updated while rest stay unchanged.
Settings.update_interval = Duration.from_millis(100)
```

Individual settings cen be assigned directly (`Settings.font = ...`) from any user script, and if supported, will be reflected
in real time instead of on only updating on app startup.

### `Setting.log_level` replaced by `level_filter_table`

The global `log_level` setting is removed.

Use `level_filter_table` instead:

```lua
-- 0.11
Settings {
  log_level = LevelFilter.Debug,
}

-- 0.12 - Works same as 0.11, applies Debug level filter to default targets
Settings.level_filter_table = LevelFilterTable.default_with(LevelFilter.Debug)

-- 0.12 - per-target control
Settings.level_filter_table = {
  omni_led = LevelFilter.Info,
  ['omni_led_lib::devices'] = LevelFilter.Debug,
}
```

See [Settings](settings.md#level-filter-table) for details.

### `devices.lua`: device IDs are plain numbers

`hid_settings` values are no longer parsed from hex strings.

Remove quotes and rest stays the same:

```lua
-- 0.11
hid_settings = {
  vendor_id = '0x1038',
  product_id = '0x161C',
  interface = '0x01',
}

-- 0.12
hid_settings = {
  vendor_id = 0x1038,
  product_id = 0x161C,
  interface = 0x01,
}
```

### User scripts: special OmniLED globals are read only

Special OmniLED variables like `Log`, `PLATFORM`, etc. can no longer be overwritten. Trying to do so will result in a
runtime error. `Settings` can be overwritten but only with settings-compatible table, trying to assign e.g. an integer
will fail.

Remove any assignments to them from your scripts.

### User scripts: integers no longer accept floats

Updated `mlua`: when a type expects an integer, passing a float is now an error instead of being silently converted.

Make sure you pass whole numbers or round them using `math.round`, `math.floor` or `math.ceil`:

```lua
-- 0.11
Widget.Text {
  text = 'Example',
  position = { x = 1.4, y = 2.3 },
  size = { width = 3.9, height = 4.4 },
}

-- 0.12
Widget.Text {
  text = 'Example',
  position = { x = 1, y = math.floor(2) },
  size = { width = math.ceil(3.9), height = math.round(4) },
}
```

### Per widget font setting

Settings now control the default font that can be overwritten per-widget.

Update settings to use `default_font` and `FontSelector.Builtin`:

```lua
-- 0.11
Settings {
  font = FontSelector.Default,
}

-- 0.12 - set base font for all widgets
Settings = {
  default_font = FontSelector.Builtin,
}
```

(Optional) Set per widget font in `scripts.lua`:

```lua
local fira_mono = FontManager:get_font(FontSelector.System {
  names = { 'FiraMono' },
  weight = Weight.Bold,
})

local widget = Widget.Text {
  text = 'Example',
  position = { x = 0, y = 0 },
  size = { width = 100, height = 40 },
  font = fira_mono,
}
```

### Default font changed to DejaVu Sans Mono

If you liked the previous FiraMono Bold font it can be downloaded from the
[previous release files](https://github.com/llMBQll/OmniLED/tree/v0.11.5/assets/fonts/FiraMono) or from
[google fonts](https://fonts.google.com/specimen/Fira+Mono).

```lua
-- 0.11
Settings {
  font = FontSelector.Default,
}

-- 0.12 - if font is installed for the user/system
Settings = {
  default_font = FontSelector.System {
    names = { 'FiraMono' },
    weight = Weight.Bold,
  },
}

-- 0.12 - if font is just placed into some directory
Settings = {
  default_font = FontSelector.Filesystem {
    path = '/path/to/FiraMono-Bold.ttf'
  },
}
```
