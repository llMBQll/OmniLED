# Audio

Audio application provides the currently selected audio output and its volume.

## Running

Audio expects one required argument - server address.

```
audio --address <ADDRESS>
```

## Audio Events

Audio application sends `AUDIO` events in two forms

1. Full update on startup or main output device   
   `AUDIO`: table
    - `IsMuted`: bool
    - `Volume`: integer
    - `Name`: string
2. Partial update on main output device's volume change  
   `AUDIO`: table
    - `IsMuted`: bool
    - `Volume`: integer
