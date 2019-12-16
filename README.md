# Measure the perpendicularity of a CNC end mill

## How to use it

Hardware:

1. 3D print the [perpendicularity tool](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/perpendicularity-tool.stl)
1. The three tallest pillars form an L shape. Let's call the top one of the L `A`, the corner `B`, and the right most one `C`.
1. Screw a short M3 bolt into each of these pillars `A`, `B`, and `C`. Don't screw them all the way in. 
1. Screw the whole thing to your spoil board using the two outer M4 holes. 
   1. The line from `A` to `B` should be parallel to the Y axis, with `B` at a higher Y coordinate than `A`. 
   1. The line from `B` to `C` should be parallel to the X axis, with `C` at a higher X coordinate than `B`.
1. Get a thin wire, wrap it around the bolt in `A`, then around the bolt in `B`, and finally around the bolt in `C`. Make sure the wire is straight. This is easy when you put some tension on a very thin wire like one or a few strands of a flexible wire.
1. Attach the Z probe:
   1. One wire should be attached to either the `A` or the `C` bolt.
   1. The other wire should be attached to your end mill, as high as you can.
   1. Make sure that when the end mill touches the wire between the bolts, the Z probe is triggered.
1. Your end mill should have the same diameter from top to bottom. I used a small piece of aluminium tubing instead of an actual end mill. You may be able to put an end mill in upside down. Or maybe you can put a small metal tube over the cutting part of your end mill. It does not need to be perfectly straight.

Software:

1. Install Python 2.7, pandas, matplotlib, statsmodels, and seaborn. (You can use [Conda](https://docs.conda.io/projects/conda/en/latest/user-guide/install/) to make this easier).
1. Download the [perpendicularity.py](https://github.com/pvdbrand/cnc-z-perpendicularity/blob/master/perpendicularity.py) script
1. Edit the settings at the top of the script (below the imports) to your needs:
   1. `marlinPort` and `marlinBaudrate` are used to connect to Marlin over the USB serial interface
   1. `probeXpos` and `probeYpos` determine the where to probe (relative to the starting position, no need to change if you use the 3D printed tool)
   1. `probeWidth` should be the diameter of your end mill
   1. `probeHeight` is how much of the end mill should be used to probe
   1. `boltHeadHeight` is the distance from the top of the bolt at `B` to the thin wire
   1. `maxBacklash` specifies the maximum measured backlashed allowed for a measurement to be included in the calculation of the end mill angle.

How to run the script:

1. Position the end mill directly on top of the bolt at `B` (the Z probe needs to be triggered)
1. Run `python perpendicularity.py`
1. Wait a couple of minutes
1. When prompted, rotate the end mill 180 degrees, and press Enter to continue. (You may need to detach and reattach the Z probe wire)
1. Wait a bit more
1. You'll now see the results. If enabled, you can also see backlash and end mill runout estimates.
