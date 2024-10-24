# Weather

Weather application provides current weather conditions at a requested place.

## Running

Weather in a specified city

```
weather --address <ADDRESS> [--interval <INTERVAL>] [--unit <UNIT>] \
    in <CITY> [--country-code <CODE>] [--administrative <ADMINISTRATIVE>]
```

Weather at specified coordinates

```
weather --address <ADDRESS> [--interval <INTERVAL>] [--unit <UNIT>] \
    at <LATITUDE> <LONGITUDE>
```

## Weather Events

`WEATHER`: table

- `Latitude`: float,
- `Longitude`: float,
- `Temperature`: float,
- `WindSpeed`: float,
- `WindDirection`: integer,
- `IsDay`: bool,
- `WeatherDescription`: string,
- `ImageKey`: string,
- `UpdateHour`: integer,
- `UpdateMinute`: integer,
- `City`: string,