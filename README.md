# Automatically calibrating a CNC

## How to use it

Hardware:

1. Build the arm:
   1. 3D print the [arm](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/arm.stl)
   1. Insert a needle in the clamp at the end
   1. Mount it to the collet nut of your router
1. Mount a single feeler gauge about 20 mm above your spoilboard.
1. Attach the Z probe:
   1. One wire should be attached to the feeler gauge.
   1. The other wire should be attached to the needle.

CNC Firmware:

The script only uses gcode to communicate with your CNC router. Any firmware that supports the required gcodes should work. It has only been tested with Marlin so far.

When using Marlin:

1. The Z probe should be wired and configured to trigger the `z_min` endstop. (It was the only way I could get `G28 Z` and `G38` to work at the same time. If yours is triggered by `z_probe`, then change `z_min` to `z_probe` in the `isZProbeTriggered` function.)
1. Enable `G38` in Marlin in `Conguration_adv.h`:
   1. Uncomment `#define G38_PROBE_TARGET`
   1. Uncomment `#define G38_PROBE_AWAY`
   1. Recompile and upload Marlin to your CNC router

Software:

1. For the simulator:
   1. Install [Rust](https://www.rust-lang.org/tools/install)
   1. In the simulator subdirectory, run `cargo build` or `cargo run`
1. For the calibration script:
   1. Install Python 2.7, pandas, matplotlib, and statsmodels (you can use [Conda](https://docs.conda.io/projects/conda/en/latest/user-guide/install/) to make this easier)
   1. Edit the settings at the top of the script (below the imports) to your needs

Running the script:

1. Position the end mill directly on top of the bolt at `B` (the Z probe needs to be triggered)
1. Open [perpendicularity.py](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/perpendicularity.py) in the Spyder IDE and run it. Alternatively, run `python perpendicularity.py` from the command line
1. Wait a couple of minutes
1. When prompted, rotate the end mill 180 degrees, and press Enter to continue. (You may need to detach and reattach the Z probe wire)
1. Wait a bit more
1. You'll now see the results. If enabled, you can also see backlash and end mill runout estimates.

## The simulator in action

See [this video on Youtube](https://www.youtube.com/watch?v=3-CxL5ajJyM).
