# Systen

System application provides information about the following system resources:

- CPU usage and temperature
- GPU usage and temperature
- memory usage

## Running

```shell
media --address <ADDRESS> [--interval <TIME_STRING>]
```

Media expects three arguments

- Required:
  - `a`/`address` - server address
- Optional:
  - `i`/`interval` - update interval  
    Default: `2sec`.
  
## Available information

This application uses [all-smi](https://github.com/lablup/all-smi) to read system information.

## System Events

System application sends the following event with the frequency set by interval option.

`SYSTEM`: table

- `Cpus`: \[CpuData\],
- `Gpus`: \[GpuData\],
- `Memory`: MemoryData | none,

CpuData

- `Name`: string,
- `Utilization`: float,
- `Temperature`: float | none,
- `PowerConsumption`: float | none,
- `Cores`: \[CoreData\],

CoreData

- `Id`: integer,
- `Utilization`: float,
- `Type`: string ("Super" | "Performance" | "Efficiency" | "Standard")

GpuData

- `Name`: string,
- `Utilization`: float,
- `Temperature`: float,
- `PowerConsumption`: float,
- `Frequency`: integer,
- `UsedMemory`: integer (value in bytes),
- `TotalMemory`: integer (value in bytes),

MemoryData

- `Used`: integer (value in bytes),
- `Total`: integer (value in bytes),
- `SwapUsed`: integer (value in bytes),
- `SwapTotal`: integer (value in bytes),
- `Utilization`: float,
