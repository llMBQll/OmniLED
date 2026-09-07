# Settings

You can fine tune behaviour of the program using the [settings file](../config/settings.lua).
All the top-level properties described below are optional and will be set to default, should any of
them be missing.

## Available Settings

- [Animation](#animation)
- [Font](#font)
- [Level Filter Table](#level-filter-table)
- [Keyboard](#keyboard)
- [Steelseries API](#steelseries-api)
- [Update Interval](#update-interval-tick-duration)

> ### Animation
>
> Animation settings apply to scrolling text or animated images.
>
> > `animation_ticks_delay`: `integer`
> >
> > Number of [ticks](#update-interval-tick-duration) after which the animation will start to
> > advance.
> >
> > _Optional_. Default: `8`
>
> > `animation_ticks_rate`: `integer`
> >
> > Number of [ticks](#update-interval-tick-duration) between consecutive animation steps.
> >
> > _Optional_. Default: `2`
>
> > Example `settings.lua` that sets scroll delay and repeat delay.
> >
> > ```lua
> > Settings.animation_ticks_delay = 4,
> > Settings.animation_ticks_rate = 1,
> > ```

> ### Font
>
> > `font`: [`FontSelector`](scripting_reference.md#fontselector)
> >
> > Set font used for rendering text on screen.
> >
> > _Optional_. Default: `Default`.
> >
> > While it's possible to load any font style - monospace fonts are highly recommended, due to the
> > simplistic nature of text rendering implementation.
>
> > Example `settings.lua` that loads default font
> >
> > ```lua
> > Settings.font = 'Default'
> > ```
>
> > Example `settings.lua` that loads font from the file system
> >
> > ```lua
> > Settings.font = {
> >   Filesystem = {
> >     path = '/path/to/my/font',
> >     font_index = 0,
> >   }
> > }
>
> > Example `settings.lua` that loads installed system font
> >
> > ```lua
> > Settings.font = {
> >   System = {
> >     names = {'FiraMono', 'Monospace'},
> >     style = 'Normal',
> >     weight = 'Bold',
> >     stretch = 'Condensed',
> >   }
> > }
> > ```

> ### Level Filter Table
>
> > `level_filter_table`: `table<string, LevelFilter>`
> >
> > _See [`LevelFilter`](scripting_reference.md#levelfilter)_
> >
> > Set a per target log level. This is useful to limit certain targets or allow more tracing from another.
> > Usually you just want the default setting `LevelFilterTable.default()` or change it across all default targets using
> > `LevelFilterTable.default_with(LevelFilter.<LEVEL>)`.
> >
> > _Optional_. Default: `LevelFilterTable.default()`
> >
> > _See [`LevelFilterTable`](scripting_reference.md#levelfiltertable)_
>
> > Example `settings.lua` that accepts debug log levels and above using default targets.
> >
> > ```lua
> > Settings.level_filter_table = 'LogFilterMap.default_with(LevelFilter.Debug)'
> > ```
>
> > Example `settings.lua` that accepts custom per target levels.
> >
> > ```lua
> > Settings.level_filter_table = {
> >   omni_led = LevelFilter.Info,
> >   omni_led_lib = LevelFilter.Trace,
> >   ['omni_led_lib::devices'] = LevelFilter.Debug,
> > }
> > ```

> ### Keyboard
>
> > `keyboard_ticks_repeat_delay`: `integer`
> >
> > Number of [ticks](#update-interval-tick-duration) of holding a key after OmniLED will start
> > repeating the key press.
> >
> > _Optional_. Default: `2`
>
> > `keyboard_ticks_repeat_rate`: `integer`
> >
> > Number of [ticks](#update-interval-tick-duration) between consecutive repeats.
> >
> > _Optional_. Default: `2`
>
> > Example `settings.lua` that sets repeat delay and repeat delay.
> >
> > ```lua
> > Settings.keyboard_ticks_repeat_delay = 4,
> > Settings.keyboard_ticks_repeat_rate = 1,
> > ```

> ### Steelseries API
>
> > `steelseries_api.address`: `string`
> >
> > Will override the default logic and use the provided api address.
> > Might be useful for non-standard setups or when the adddress parsing logic is failing
> > for any reason.
> >
> > _Optional_. Default: `nil`
>
> > `steelseries_api.config_path`: `string`
> >
> > Will override the default logic and use the provided coreProps.json path.
> > Might be useful for non-standard setups or when the logic is failing for any reason.
> >
> > _Optional_. Default:
> >
> > - Linux:   `nil`,
> > - macOS:   `"/Library/Application Support/SteelSeries Engine 3/coreProps.json"`,
> > - Windows: `"%PROGRAMDATA%/SteelSeries/SteelSeries Engine 3/coreProps.json"`
>
> > `steelseries_api.enabled`: `bool`
> >
> > Enables or disables the Steelseries API integration.
> >
> > _Optional_. Default:
> >
> > - Linux:   `false`,
> > - macOS:   `true`,
> > - Windows: `true`
>
> > `steelseries_api.register_heartbeat`: `bool`
> >
> > Enables or disables the heartbeat event. SteelSeries API expects reugular updates, or
> > it deems the sender to be disconnected. To prevent this, if OmniLED doesn't send any
> > render requests for some time it will automatically send a heartbeat event to remind
> > SteelSeries API it still exists.
> >
> > _Optional_. Default: `true`
>
> > `steelseries_api.deinitialize_timeout`: `Duration`
> >
> > Register to SteelSeries APi with a given `deinitialize_timeout`. This controls how
> > soon the API will forget about OmniLED if no events are sent. This setting also impacts
> > how often the heartbeats are sent (if enabled) to keep the connection alive.  
> > It accepts a range [1s, 60s] inclusive.
> >
> > _Optional_. Default: `Duration.from_secs(15)`
>
> > Example `settings.lua` that sets up the SteelSeries API.
> >
> > ```lua
> > Settings.steelseries_api.address = "some_address",
> > Settings.steelseries_api.config_path = "some_path",
> > Settings.steelseries_api.enabled = true,
> > Settings.steelseries_api.register_heartbeat = true,
> > Settings.steelseries_api.deinitialize_timeout = Duration.from_secs(7),
> > ```

> ### Update interval (Tick Duration)
>
> > `update_interval`: `integer`
> >
> > This setting will define how ofter the server will process events and render updates on the
> > screen. Lower interval will increase responsiveness at the cost of the CPU usage. Update
> > interval (or tick duration) is defined in milliseconds.
> >
> > _Optional_. Default: `100`
>
> > Example `settings.lua` that sets update interval to `50`.
> >
> > ```lua
> > Settings.update_interval = 50,
> > ```
