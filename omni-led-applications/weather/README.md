# Weather

Weather application provides current weather conditions at a requested place.

## Running

**Weather in a specified city:**

```shell
weather <COMMON-OPTIONS> \
    in <CITY> [--country-code <CODE>] [--administrative <ADMINISTRATIVE>]
```

2-letter country code and administrative - (state, province, voivodeship, etc.) can be provided to narrow down the
search in case multiple cities have the provided name

**Weather at specified coordinates:**

```shell
weather <COMMON-OPTIONS> \
    at <LATITUDE> <LONGITUDE>
```

---

**Common Options:**

- `--address`: server address
- `--interval`: interval between trying to refresh weather information
- `--temperature-unit`: unit in which temperature will be reported
- `--wind-speed-unit`: unit in which wind speed will be reported

## Weather Events

`WEATHER`: table

- `City`: string (for now will be 'N/A' when requesting weather at coordinates rather than in a city),
- `ImageKey`: string (index into WEATHER.Images array that depicts current weather conditions),
- `IsDay`: bool,
- `Latitude`: float,
- `Longitude`: float,
- `Temperature`: float (in TemperatureUnits),
- `TemperatureUnit`: string,
- `UpdateHour`: integer (time of temperature measurement),
- `UpdateMinute`: integer (time of temperature measurement),
- `WeatherDescription`: string,
- `WindDirection`: integer (wind direction in degrees [0 - 360)),
- `WindSpeed`: float (in WindSpeedUnits),
- `WindSpeedUnit`: string

## Roadmap

- [x] Current weather conditions
- [ ] Weather forecast
- [ ] Mapping coordinates to city name
