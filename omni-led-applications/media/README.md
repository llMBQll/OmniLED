# Media

Media application provides information about currently playing media, e.g. title, artist, duration etc.

## Running

```
media --address <ADDRESS> [--mode <MODE>] [--map <MAPPING>...]
```

Media expects three arguments

- Required:
    - `a`/`address` - server address
- Optional:
    - `m`/`mode` - reporting mode - `individual`, `focused` or `both`.   
      Default: `both`.  
      More info [here](#reporting-mode).
    - `map` - map input application name to an event name, e.g. `--map "my_app_name=APP"`. Can be passed multiple
      times. Target name must be an uppercase alphanumeric string, that can contain underscores and cannot start with a
      number.   
      Default: `[]`.  
      More info [here](#application-name-mapping).

## Reporting mode

### Individual

Send separate events for each currently playing application. Each application will have its own event name as
described [here](#application-name-mapping).

### Focused

Send events only for the currently focused media source. The focused state is determined by your operating system.  
All updates will be sent with event name `MEDIA`, regardless of source application.

### Both

Report events in both ways - individual per application and combined for currently focused application.

## Application name mapping

When sending events in [individual](#individual) mode, application names will be mapped to event names.  
If mapping was provided as a command line parameter, then it will use the target name from that mapping.  
If mapping was not provided, source application name will be converted in the following manner:

- If name starts with a digit it will be prefixed with an underscore.
- All ascii letters will be converted to uppercase.
- All non-alphanumeric characters will be converted to underscores.

Examples:

- `my_app_name.exe` > `MY_APP_NAME_EXE`
- `123fourfive` > `_123FOURFIVE`

> To see actual resulting mapping check `data/media/logging.log`. An entry will be logged every time a new source is
> detected.
>
> Example log entry
`[2024-09-28 15:00:45:425][INFO][media] Mapped '308046B0AF4A39CB' to '_308046B0AF4A39CB'`

## Media Events

Media sends a single type of event, and its name depends on the selected [mode](#reporting-mode).

> There is a discrepancy in event frequency between current implementations on Windows and Linux operating systems.  
> On Windows the interval seems to be around 4 seconds and on Linux it's a fixed update interval of 1 second.

> Availability of event fields depends entirely on the media source. Be sure to check if a field is present when
> handling media events.

`MEDIA` or `<MAPPED_NAME>`: table

- `Artist`: string,
- `Title`: string,
- `Progress`: integer (value in milliseconds),
- `Duration`: integer (value in milliseconds),
- `Playing`: bool,