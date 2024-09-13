# Clock

Clock application provides the current time and date, as well as day and month names.

## Running

Clock expects one required argument - server address.

```
clock --address <ADDRESS>
```

## Clock Events

Clock application sends `CLOCK` events in two forms

1. Day and month names on startup  
   `CLOCK`: table
    - `DayNames`: table
    - `MonthNames`: table
2. Current time every second  
   `CLOCK`: table
    - `Hours`: integer [0-23]
    - `Minutes`: integer [0-59]
    - `Seconds`: integer [0-59]
    - `MonthDay`: integer [1-31]
    - `WeekDay`: integer [1-7]
    - `Month`: integer [1-12]
    - `Year`: integer