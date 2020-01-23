# Automatically calibrating a CNC

## What's this?

The repository contains a Python script to automatically determine how perpendicular the Z axis of a CNC machine is, using just a feeler gauge and a touch probe.

This repository also contains a simulator (written in Rust) to make sure that the movements and calculations in the Python script are actually correct.

## How to use it

### Hardware

1. Build the arm:
   1. 3D print the [arm](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/parts/arm.stl)
   1. Insert a needle in the clamp at the end
   1. Insert an M3 bolt in the hole close to the collet nut hole and connect the needle to this bolt with a piece of wire
   1. Mount the arm to the collet nut of your router and use another M3 bolt to secure it in place
1. Build the feeler gauge holder:
   1. 3D print the [holder](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/parts/gauge_holder.stl)
   1. Wrap a piece of wire around the middle of feeler gauge
   1. Put a feeler gauge on top of the holder, put the lid on top, and screw it together with 4 M3 bolts
   1. Bolt the assembly to your spoil board
1. Attach the Z probe:
   1. One wire should be attached to the wire that is connected to the feeler gauge
   1. The other wire should be attached to the M3 bolt that is connected to the needle

### CNC Firmware

The script only uses gcode to communicate with your CNC router. Any firmware that supports the required gcodes should work. It has only been tested with Marlin so far.

When using Marlin:

1. The Z probe should be wired and configured to trigger the `z_min` endstop. (It was the only way I could get `G28 Z` and `G38` to work at the same time. If yours is triggered by `z_probe`, then change `z_min` to `z_probe` in the `isZProbeTriggered` function.)
1. Enable `G38` in Marlin in `Conguration_adv.h`:
   1. Uncomment `#define G38_PROBE_TARGET`
   1. Uncomment `#define G38_PROBE_AWAY`
   1. Recompile and upload Marlin to your CNC router

### Software

1. For the simulator:
   1. Install [Rust](https://www.rust-lang.org/tools/install)
   1. In the simulator subdirectory, run `cargo build` or `cargo run`
1. For the calibration script:
   1. Install Python 2.7, pandas, matplotlib, and statsmodels (you can use [Conda](https://docs.conda.io/projects/conda/en/latest/user-guide/install/) to make this easier)
   1. Edit the settings at the top of the script (below the imports) to your needs

### Running the script

1. Put the arm on the collet nut at a 45 degree angle, pointing towards positive X and positive Y
1. Position the needle on top of the feeler gauge, about 5mm to the right of the tip
1. Open [feeler_gauge.py](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/feeler_gauge.py) in the Spyder IDE and run it. Alternatively, run `python feeler_gauge.py` from the command line

## The simulator in action

See [this video on Youtube](https://www.youtube.com/watch?v=3-CxL5ajJyM).
