# Applications

Applications are plugins that provide the data. OmniLED than collects that data and draws it on the screen.
There are some [built-in applications](#built-in-applications) provided, but it's possible to load any 
[custom application](#custom-applications) that provides the data you need.

## Built-in Applications

OmniLED comes some applications pre-installed. Each one comes with its own README file that describes their
usage and purpose.

- [audio](../oled-applications/audio/README.md)
- [clock](../oled-applications/clock/README.md)
- [media](../oled-applications/media/README.md)
- [weather](../oled-applications/weather/README.md)

## Custom Applications

Custom applications may be written in any language as long as they implement
the [gRPC interface](../oled-api/proto/plugin.proto).

## Loading Applications

There are 2 ways for applications to be started.

### Registering in `applications.lua`

The first way is to register applications inside the `applications.lua` file. This allows the OmniLED
process to manage the application's lifetime — starting it when OmniLED starts and stopping it when
OmniLED shuts down. Additionally, you can define command line arguments that will be passed to theapplication.

To load an application use the global [`load_app`](lua_interfaces.md#load_app) function. This allows for the
script to set command line arguments.

> _Note: This is the preferred approach when application is only supposed to provide data to OmniLED as it's simply the
> easier option._

### Starting the Application Independently

The second way is to start the application independently. In this way the client application does not depend on the
OmniLED being launched.

> _Note: This is the preferred approach when application is doing multiple things and sending events to OmniLED just
happens to be one of them._

### Accessing the Server Address

Regardless of the approach, the client application need to know the server address. OmniLED provides two ways to get
the server address.

#### Using the Global `SERVER` Variable

If the application is registered in `applications.lua`, it can access the
[`SERVER`](lua_interfaces.md#server) constant, which contains the
server address. This eliminates the need for the application to determine the server address manually.

> Example `applications.lua` file:
> ``` lua
> load_app {
>   path = get_default_path('my_application'),
>   args = {
>     '--address', SERVER.Address,
>   }
> }
> 
> local path = PLATFORM.ApplicationsDir 
>              .. PLATFORM.PathSeparator 
>              .. 'my_other_application'
>              .. PLATFORM.ExeSuffix
> load_app {
>   path = path,
>   args = {
>     '--ip', SERVER.Ip,
>     '--port', SERVER.Port,
>   }
> }
> ```
> 
> In the above applications there 2 applications loaded: `my_application` and `my_other_application`.  
> They both can be found in the default application directory, but for the first one
> [`get_default_path`](lua_interfaces.md#get_default_path) function was used, and for the other the path was
> constructed manually using the [`PLATFORM`](lua_interfaces.md#platform) constants.  
> Also the application received different command line arguments, to send exactly what they expect. For built-in
> applications' arguments refer to this [paragraph](#built-in-applications).

#### Reading `server.json`

For standalone applications, the OmniLED server automatically generates a file named `server.json`
in the `data` directory. This file provides all the necessary details for the application to connect to the server.
Here’s an example of a `server.json` file:

``` json
{
  "address": "127.0.0.1:44631",
  "ip": "127.0.0.1",
  "port": 44631,
  "timestamp": 1726234620609
}
```
