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

## Logging

Weather application will log relevant info to so you can make sure it's providing information for the correct location.

Example of matching the city successfully:

<!-- markdownlint-disable MD013 -->
```log
[2026-05-09 15:05:16:167][DEBUG][plugin::weather] Found 'Warsaw': GeocodingData { latitude: 52.22977, longitude: 21.01178, country_code: "PL", admin1: Some("Mazovia"), admin2: Some("Warszawa"), admin3: Some("Warsaw"), admin4: None }
```
<!-- markdownlint-enable MD013 -->

---

Example of a typo in the `administrative` option resulting in no exact matches:

<!-- markdownlint-disable MD013 -->
```log
[2026-05-09 14:59:37:268][ERROR][plugin::omni_led_api::logging] panicked at omni-led-applications/weather/src/main.rs:100:17:
Didn't match any entries for Name { city: "Warsaw", country_code: Some("PL"), administrative: Some("Mazovian") }. Entries:
  [1/10] GeocodingData { latitude: 52.22977, longitude: 21.01178, country_code: "PL", admin1: Some("Mazovia"), admin2: Some("Warszawa"), admin3: Some("Warsaw"), admin4: None }
  [2/10] GeocodingData { latitude: 41.2381, longitude: -85.85305, country_code: "US", admin1: Some("Indiana"), admin2: Some("Kosciusko"), admin3: Some("Wayne Township"), admin4: None }
  [3/10] GeocodingData { latitude: 42.74006, longitude: -78.13279, country_code: "US", admin1: Some("New York"), admin2: Some("Wyoming"), admin3: Some("Town of Warsaw"), admin4: None }
  [4/10] GeocodingData { latitude: 38.24308, longitude: -93.38187, country_code: "US", admin1: Some("Missouri"), admin2: Some("Benton"), admin3: Some("South Lindsey Township"), admin4: None }
  [5/10] GeocodingData { latitude: 38.7834, longitude: -84.90162, country_code: "US", admin1: Some("Kentucky"), admin2: Some("Gallatin"), admin3: None, admin4: None }
  [6/10] GeocodingData { latitude: 37.95874, longitude: -76.75801, country_code: "US", admin1: Some("Virginia"), admin2: Some("Richmond"), admin3: None, admin4: None }
  [7/10] GeocodingData { latitude: 34.99933, longitude: -78.0911, country_code: "US", admin1: Some("North Carolina"), admin2: Some("Duplin"), admin3: Some("Warsaw Township"), admin4: None }
  [8/10] GeocodingData { latitude: 40.35921, longitude: -91.4346, country_code: "US", admin1: Some("Illinois"), admin2: Some("Hancock"), admin3: Some("Warsaw Township"), admin4: None }
  [9/10] GeocodingData { latitude: 40.33535, longitude: -82.00681, country_code: "US", admin1: Some("Ohio"), admin2: Some("Coshocton"), admin3: Some("Jefferson Township"), admin4: None }
  [10/10] GeocodingData { latitude: 44.24941, longitude: -93.39383, country_code: "US", admin1: Some("Minnesota"), admin2: Some("Rice"), admin3: Some("Warsaw Township"), admin4: None }
```
<!-- markdownlint-enable MD013 -->

## Roadmap

- [x] Current weather conditions
- [ ] Weather forecast
- [ ] Mapping coordinates to city name
