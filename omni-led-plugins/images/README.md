# Images

Images plugin reads images from disk and sends them to OmniLED. It supports both static and
animated images.

## Running

```shell
images [--image '<NAME> <PATH> [--format <FORMAT>] [--threshold <THRESHOLD>]']...
```

Images takes a single optional argument (cay be specified multiple times)

- Optional:
  - `-i`/`--image` - loaded image options.  
      This option can be specified multiple times and it a **single string** as it's argument with
      image load options:
    - `<NAME>` - This name will be used as variable name in user scripts.  
          _This is a positional argument and shall always be specified as a first argument._
    - `<PATH>` - Path to an image file on disk.  
          _This is a positional argument and shall always be specified as a second argument._
    - `-f`/`--format` - Image extension used as a hint for loading images when the format cannot
          automatically be deduced from the file contents.

### Example

In this example `images` will load 2 images from disk.

```lua
load_plugin {
  path = get_default_plugin_path('images'),
  args = {
    '--image', 'MyImage /path/to/my_gif --format gif',
    '--image', 'MyOtherImage "C:\\path\\to\\other image.png"',
  }
}
```

## Images Events

Images plugin sends a single event with all loaded images with names specified as program
arguments.

`IMAGES`: table

- `<NAME_1>`: `Image`,  
  ...
- `<NAME_N>`: `Image`,
