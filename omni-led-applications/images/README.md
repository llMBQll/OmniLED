# Images

Images application reads images from disk and converts them to a black and white image before
sending it to OmniLED server.

## Running

```
images --address <ADDRESS> [--image '<NAME> <PATH> [--format <FORMAT>] [--threshold <THRESHOLD>]']...
```

Images expects two arguments

- Required:
    - `a`/`address` - server address
- Optional:
    - `-i`/`--image` - loaded image options.  
      This option can be specified multiple times and it a **single string** as it's argument with
      image load options:
        - `<NAME>` - This name will be used as variable name in user scripts.  
          _This is a positional argument and shall always be specified as a first argument._
        - `<PATH>` - Path to an image file on disk.  
          _This is a positional argument and shall always be specified as a second argument._
        - `-f`/`--format` - Image extension used as a hint for loading images when path doesn't
          contain an extension.
        - `-t`/`--threshold` - Threshold that will be used when converting the image to black and
          white. Values with brightness lower than threshold will be black, and above or equal to
          threshold will be white.  
          Range: [0, 255].  
          Default: 128

### Example

In this example `images` will load 2 images from disk.

```lua
load_app {
    path = get_default_path('images'),
    args = {
        '--address', SERVER.Address,
        '--image', 'MyImage /path/to/my_image --format jpg --threshold 77',
        '--image', 'MyOtherImage "C:\\path\\to\\other image.png" --threshold 159',
    }
}
```

## Images Events

Images applications sends a single event with all loaded images with names specified as program
arguments.

`IMAGES`: table

- `<NAME_1>`: `OLEDImage`,  
  ...
- `<NAME_N>`: `OLEDImage`,