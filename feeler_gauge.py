#!/usr/bin/env python2
# -*- coding: utf-8 -*-

import math, sys
import pandas as pd
import matplotlib.pyplot as plt
import statsmodels.api as sm
import seaborn as sns

from marlin import Marlin

useSimulator = True

simulator = {
    'executable': '/home/peter/github/cnc-z-perpendicularity/simulator/target/debug/simulator',
    'working_directory': '/home/peter/github/cnc-z-perpendicularity/simulator',
    'fast': False,
}

marlinPort = "/dev/serial/by-id/usb-Arduino__www.arduino.cc__0042_85531303231351E0E181-if00"
marlinBaudrate = 250000

safeHeight = 10.0  # mm
safeDistance = 10.0 # mm
probeHeight = -10.0 # mm, should be negative

zSpeed = 3 # mm/s
xySpeed = 8 # mm/s
probeSpeed = 1 # mm/s

###############################################################################

pd.set_option('display.max_columns', 10)
pd.set_option('display.width', 125)

###############################################################################

marlin = Marlin(simulator if useSimulator else None)
marlin.connect(marlinPort, marlinBaudrate)

if useSimulator:
    marlin.send('M800 A0 B0')
    marlin.send('M801 A0 B0 R0')
    marlin.send('M802 A0 B0 O150')
    marlin.send('G1 X350')
    marlin.home()

if not marlin.isZProbeTriggered():
    print 'Error: Z probe is not triggered.'
    print 'Make sure the router is in the correct starting position, and that the Z probe is triggered.'
    sys.exit(0)

marlin.enableSteppers()
marlin.setPosition(0, 0, 0)
marlin.go(0, 0, safeHeight, mm_per_second=zSpeed)
marlin.waitUntilStopped()

if marlin.isZProbeTriggered():
    print 'Error: Z probe is still triggered after trying to move up. PLease check your probe.'
    sys.exit(0)

circle = []

# find the outline of the feeler gauge
points = \
    [(dx, -safeDistance) for dx in range(0, int(round(safeDistance)))] + \
    [(dx, safeDistance) for dx in range(0, int(round(safeDistance)))]

measurements = []
first = True
for side in [-1, 1]:
    for i in range(-2, 2+1):
        x = side * -i * 2 + 2 * 2
        y = side * safeDistance
        
        if first:
            marlin.go(x, y, safeHeight, mm_per_second=xySpeed)
            marlin.go(x, y, probeHeight, mm_per_second=zSpeed)
            first = False
            
        marlin.go(x, y, probeHeight, mm_per_second=xySpeed, wait=True)
        assert(not marlin.isZProbeTriggered())
    
        x, y, _ = marlin.probe(x, 0, probeHeight, mm_per_second=probeSpeed, towards=True)
        measurements += [{'x': x, 'y': y, 'ok': marlin.isZProbeTriggered()}]

        marlin.go(x, y, probeHeight, mm_per_second=xySpeed)

    marlin.go(x, y, safeHeight, mm_per_second=zSpeed)

measurements = pd.DataFrame(measurements)

centerline = measurements.groupby('x')[['y']].mean().reset_index()
centerline['c'] = 1.0

model = sm.OLS(centerline.y, centerline[['c', 'x']]).fit()
center_y_at_x0 = model.params['c'] + model.params['x'] * 0.0
center_y_safe = model.params['c'] + model.params['x'] * -safeDistance

marlin.go(-safeDistance, center_y_safe, safeHeight, mm_per_second=xySpeed)
marlin.go(-safeDistance, center_y_safe, probeHeight, mm_per_second=zSpeed, wait=True)
assert(not marlin.isZProbeTriggered())

x, y, _ = marlin.probe(0, center_y_at_x0, probeHeight, mm_per_second=probeSpeed, towards=True)
x += 1.0
y += model.params['x'] * 1.0

marlin.go(-safeDistance, center_y_safe, probeHeight, mm_per_second=xySpeed)
marlin.go(-safeDistance, center_y_safe, safeHeight, mm_per_second=zSpeed)
marlin.go(x, y, safeHeight, mm_per_second=xySpeed, wait=True)
assert(not marlin.isZProbeTriggered())

x, y, z = marlin.probe(x, y, probeHeight, mm_per_second=probeSpeed, towards=True)
circle += [{'x': x, 'y': y, 'z': z, 'angle': 0.0}]

marlin.go(x, y, safeHeight, mm_per_second=zSpeed, wait=True)
assert(not marlin.isZProbeTriggered())

# Now rotate the spindle
marlin.go(0, -safeDistance, safeHeight, mm_per_second=xySpeed)
marlin.go(0, -safeDistance, probeHeight, mm_per_second=zSpeed, wait=True)
assert(not marlin.isZProbeTriggered())

_, _, _ = marlin.probe(0, 0, probeHeight, mm_per_second=probeSpeed, towards=True)
assert(marlin.isZProbeTriggered())
