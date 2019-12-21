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
    'fast': True,
}

marlinPort = "/dev/serial/by-id/usb-Arduino__www.arduino.cc__0042_85531303231351E0E181-if00"
marlinBaudrate = 250000

probeYpos = 20
probeXpos = 20
probeWidth = 4
probeHeight = 15
boltHeadHeight = 7.5
maxBacklash = 0.06

measurementsCsvFile = None # 'measurements.csv'
showBacklashHistogram = False
showAngleFit = False
showRunoutFit = False

###############################################################################

pd.set_option('display.max_columns', 10)
pd.set_option('display.width', 125)

###############################################################################

marlin = Marlin(simulator if useSimulator else None)
marlin.connect(marlinPort, marlinBaudrate)

if useSimulator:
    marlin.send('M800 A-5 B-5')
    marlin.send('M801 A1 B2 R0')
    marlin.send('M802 A0.5 B0.25 O0')
    marlin.send('G1 X497 Y255')
    marlin.home()

if not marlin.isZProbeTriggered():
    print 'Error: Z probe is not triggered.'
    print 'Make sure the router is in the correct starting position, and that the Z probe is triggered.'
    sys.exit(0)

safeHeight = boltHeadHeight + 2

marlin.enableSteppers()
marlin.setPosition(0, 0, boltHeadHeight)
marlin.go(0, 0, safeHeight, mm_per_second=3)
marlin.waitUntilStopped()

if marlin.isZProbeTriggered():
    print 'Error: Z probe is still triggered after trying to move up. PLease check your probe.'
    sys.exit(0)

measurements = []
for rotation in [0, 180]:
    for axis in ['x', 'y']:
        for side in [-1, 1]:
            if axis == 'x':
                x = probeXpos
                px = probeXpos
                dx = 0
                
                y = side * (probeWidth + 3)
                py = side * -10
                dy = side * 1
            else:
                x = side * (probeWidth + 3)
                px = side * -10
                dx = side * 1
                
                y = probeYpos
                py = probeYpos
                dy = 0
            
            marlin.go(x + dx, y + dy, safeHeight)
            for z in reversed(range(-2, -probeHeight, -1)):
                marlin.go(x + dx, y + dy, z, mm_per_second=3, wait=True)
                assert(not marlin.isZProbeTriggered())

                for attempt in range(3):
                    x, y, _ = marlin.probe(px, py, z, mm_per_second=1, towards=True)
                measurements += [{'x': x, 'y': y, 'z': z, 
                                  'axis': axis, 'side': side, 'rotation': rotation,
                                  'towards': True, 'ok': marlin.isZProbeTriggered()}]

                if not useSimulator:
                    for attempt in range(3):
                        x, y, _ = marlin.probe(x + dx, y + dy, z, mm_per_second=1, towards=False)
                    measurements += [{'x': x, 'y': y, 'z': z, 
                                      'axis': axis, 'side': side, 'rotation': rotation,
                                      'towards': False, 'ok': not marlin.isZProbeTriggered()}]
                
                marlin.go(x + dx, y + dy, z, mm_per_second=3, wait=True)
                assert(not marlin.isZProbeTriggered())
                
            marlin.go(x + dx, y + dy, safeHeight, mm_per_second=3)

    marlin.go(0, 0, safeHeight)
    if rotation == 0:
        if useSimulator:
            marlin.send('M801 R180')
        else:
            print 'Rotate the end mill 180 degrees and press Enter to continue...'
            raw_input()

marlin.go(0, 0, safeHeight)
marlin.go(0, 0, boltHeadHeight, mm_per_second=3)    
marlin.disableSteppers(x=True, y=True, z=True)

if useSimulator:
    marlin.close()
    
measurements = pd.DataFrame(measurements)
if measurementsCsvFile is not None:
    measurements.to_csv(measurementsCsvFile, index=False)

###############################################################################

if 0:
    axis = 'x'; rows = measurements[measurements['axis'] == axis]
    axis = 'y'; rows = measurements[measurements['axis'] == axis]

for axis, rows in sorted(measurements.groupby('axis')):
    column = 'x' if axis == 'y' else 'y'
    
    invalidZ = set(rows[~rows['ok']].z)
    if len(invalidZ) > 0:
        print 'Warning: %s axis: ignoring possibly invalid measurements at these Z coordinates: %s' % (axis, str(list(sorted(invalidZ))))
        rows = rows[~rows['z'].isin(invalidZ)]
        
    sides = None
    for rotation in [0, 180]:
        df = rows[rows.rotation == rotation].set_index('z')
        df = pd.DataFrame({
            'towardsLeft'  : df[(df.side < 0) & df.towards][column],
            'awayLeft'     : df[(df.side < 0) & ~df.towards][column],
            'towardsRight' : df[(df.side > 0) & df.towards][column],
            'awayRight'    : df[(df.side > 0) & ~df.towards][column]
        }).sort_index()
        
        df['left'] = df['towardsLeft'] # df[['towardsLeft', 'awayLeft']].mean(axis=1)
        df['right'] = df['towardsRight'] # df[['towardsRight', 'awayRight']].mean(axis=1)
        df['backlashLeft'] = (df['towardsLeft'] - df['awayLeft']).abs()
        df['backlashRight'] = (df['towardsRight'] - df['awayRight']).abs()            
        
        if sides is None:
            sides = df
        else:
            sides = sides.merge(df, left_index=True, right_index=True, suffixes=('_0', '_180'))
    
    if showBacklashHistogram:
        backlash = sides[[c for c in sides if c.startswith('backlash')]].stack()
        pd.DataFrame({'Histogram of backlash along %s axis' % axis: backlash}).hist(bins=20)
        plt.show()

    backlashTooLarge = (sides[[c for c in sides if c.startswith('backlash')]] > maxBacklash).any(axis=1)
    backlashTooLargeZ = set(backlashTooLarge[backlashTooLarge].index)
    if len(backlashTooLargeZ) > 0:
        print 'Warning: %s axis: ignoring measurements with large backlash at these Z coordinates: %s' % (axis, str(list(sorted(backlashTooLargeZ))))
        sides = sides[~sides.index.isin(backlashTooLargeZ)]

    sides = sides[[c for c in sides if c.startswith('left') or c.startswith('right')]].copy()

    sides['center_0'] = sides[['left_0', 'right_0']].mean(axis=1)
    sides['center_180'] = sides[['left_180', 'right_180']].mean(axis=1)

    tool = pd.DataFrame({'center': sides[['center_0', 'center_180']].mean(axis=1), 'z': sides.index, 'c': 1.0})
    toolModel = sm.OLS(tool['center'], tool[['c', 'z']]).fit()
    toolSlope = toolModel.params['z']
    
    print 'Angle around the %s axis:        %8.2f degrees off perpendicular' % (axis, math.degrees(math.atan2(toolSlope, 1)))
    if showAngleFit:
        sns.regplot(sides.index.values, sides[['center_0', 'center_180']].mean(axis=1).values)
        plt.show()
    
    runout = pd.DataFrame({
            'radius': (sides['center_0'] - sides['center_180']) / 2.0, 
            'z': sides.index, 'c': 1.0})
    runoutModel = sm.OLS(runout['radius'], runout[['c', 'z']]).fit()
    runoutSlope = runoutModel.params['z']
    
    print 'Runout angle around the %s axis: %8.2f degrees' % (axis, math.degrees(math.atan2(runoutSlope, 1)))
    if showRunoutFit:
        sns.regplot(runout.z.values, runout.radius.values)
        plt.show()
