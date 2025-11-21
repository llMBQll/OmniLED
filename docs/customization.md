# Customization

All files with custom settings can be found in the `config` directory in the installation
directory. This should be `~/.config/OmniLED/config` on Linux, or
`C:\Users\<USER>\AppData\Roaming\OmniLED\config` on Windows.  
It can also be accessed by pressing the `Config` menu on the tray icon after the application is
running.

> You may find an example configuration in the [`config`](../config/). This will also be the
> default installed config, so make sure to adjust the user scripts work with your device.

The configuration is split into 4 categories.

## Applications

> File `config/applications.lua`

In this file you can set which applications will be started by OmniLED and what command line
arguments it will pass to them.

See [applications](applications.md).

## Devices

> File `config/devices.lua`

This file stores all OLED device configurations.

See [devices](devices.md).

## Settings

> File `config/settings.lua`

This file stores global OmniLED settings.

See [settings](settings.md).

## User Scripts

> File `config/user_scripts.lua`

Last, but not least, the biggest customization point of OmniLED. With user scripts you can set what
happens on the screen of any device at any time.

See [user scripts](user_scripts.md).

---

> _Note: All configuration files are loaded on startup and will any changes will only be visible
> after restarting OmniLED._

> _Note: When running the development build (`dev` feature enabled), `config` directory will be set
> to `./config`, relative to cargo workspace root. This allows to do testing without affecting the
> already running production app._
