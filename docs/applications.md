# Applications

[applications.lua](../config/applications.lua) file defines all applications that will be started when main applications
starts. It also supports defining command line arguments that will be passed when starting applications.

> _Note: Applications can be started independently and still send events to the server.  
Registering in `applications.lua` is handy when application's sole purpose is to send events to SteelseriesOLED._

## Built-in Applications

Steelseries OLED comes some applications pre-installed. Each one comes with its own README file that describes their
usage.

- [audio](../oled-applications/audio/README.md)
- [clock](../oled-applications/clock/README.md)
- [media](../oled-applications/media/README.md)
- [weather](../oled-applications/weather/README.md)

## External Applications

Any client that implements the [gRPC interface](../oled-api/proto/plugin.proto) can send events to the server.

## Server Address

If an application wants to send events to Steelseries OLED it has to know the server address. There are two options:

1. Register application using `applications.lua` file and use global variable `SERVER` to pass server address.
2. Inside your application, read `server.json` file that is created in `data` directory. There you will find the server
   address as well as a UNIX timestamp of when the server was started.  
   **Example `server.json` file**
    ```json
    {
      "address": "127.0.0.1:44631",
      "ip": "127.0.0.1",
      "port": 44631,
      "timestamp": 1726234620609
    }
    ```