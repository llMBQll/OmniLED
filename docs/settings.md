# Settings

You can fine tune behaviour of the program using the [settings file](../config/settings.lua).
All the top-level properties described below are optional and will be set to default, should any of
them be missing.

## Available Settings

- [Animation](#animation)
- [Font](#font)
- [Log Level](#log-level)
- [Keyboard](#keyboard)
- [Server Port](#server-port)
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
> > ```lua
> > Settings {
> >   animation_ticks_delay = 4,
> >   animation_ticks_rate = 1,
> > }
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
> > ```lua
> > Settings {
> >   font = 'Default'
> > }
> > ```
>
> > Example `settings.lua` that loads font from the file system
> > ```lua
> > Settings {
> >   font = {
> >     Filesystem = {
> >       path = '/path/to/my/font',
> >       font_index = 0,
> >     }
> >   }
> > }
>
> > Example `settings.lua` that loads installed system font
> > ```lua
> > Settings {
> >   font = {
> >     System = {
> >       names = {'FiraMono', 'Monospace'},
> >       style = 'Normal',
> >       weight = 'Bold',
> >       stretch = 'Condensed',
> >     }
> >   }
> > }
> > ```

> ### Log Level
>
> > `log_level`: [`LogLevel`](scripting_reference.md#loglevel)
> >
> > Set minimum required severity of messages to be logged.
> >
> > _Optional_. Default: `Info`
>
> > Example `settings.lua` that accepts debug log levels and above.
> > ```lua
> > Settings {
> >   log_level = 'Debug'
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
> > ```lua
> > Settings {
> >   keyboard_ticks_repeat_delay = 4,
> >   keyboard_ticks_repeat_rate = 1,
> > }
> > ```

> ### Server Port
>
> > `server_port`: `integer`
> >
> > Select on which port the server will receive events from applications. When setting port `0`,
> > OS will select any available port. For any other value, server will try to bind to the specified
> > port and exit the application if it's not available.
> >
> > _Optional_. Default: `0`
>
> > Example `settings.lua` that sets server port to be assigned to the first available port.
> > ```lua
> > Settings {
> >   server_port = 0,
> > }
> > ```
>
> > Example `settings.lua` that sets server port to a fixed port.
> > ```lua
> > Settings {
> >   server_port = 1234,
> > }
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
> > ```lua
> > Settings {
> >   update_interval = 50,
> > }
> > ```