# Audio

Audio application provides the currently selected audio devices names and states.

## Running

Audio expects one required argument - server address.

```shell
audio --address <ADDRESS>
```

## Audio Events

Audio application sends `AUDIO` events in two forms:

1. Full update for both devices on startup and on main input/output device change  
   `AUDIO`: table
    - `Input`: table
        - `Connected`: bool
        - `IsMuted`: bool
        - `Volume`: integer
        - `Name`: string
    - `Output`: table
        - `Connected`: bool
        - `IsMuted`: bool
        - `Volume`: integer
        - `Name`: string

   > `Connected` field tells if the device is actually connected or not. If `Connected` is `false` then rest of the data
   > for the relevant device type is filled with dummy values.

2. Partial update on main input/output device's volume change  
   `AUDIO`: table
    - `Input`: table
        - `IsMuted`: bool
        - `Volume`: integer
    - `Output`: table
        - `IsMuted`: bool
        - `Volume`: integer

> Note that the `Input` and `Output` fields are always sent as two separate events.
