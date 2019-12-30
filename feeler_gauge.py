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

def find_center():
    global marlin
    
    cx, cy, _ = marlin.getPosition()
    measurements = []
    first = True
    for side in [-1, 1]:
        for i in range(-2, 2+1):
            x = side * -i * 2 + 2 * 2 + cx
            y = side * safeDistance + cy
            
            if first:
                marlin.go(x, y, safeHeight, mm_per_second=xySpeed)
                marlin.go(x, y, probeHeight, mm_per_second=zSpeed)
                first = False
                
            marlin.go(x, y, probeHeight, mm_per_second=xySpeed, wait=True)
            assert(not marlin.isZProbeTriggered())
        
            x, y, _ = marlin.probe(x, cy, probeHeight, mm_per_second=probeSpeed, towards=True)
            measurements += [{'x': x, 'y': y, 'ok': marlin.isZProbeTriggered()}]
    
            marlin.go(x, y, probeHeight, mm_per_second=xySpeed)
    
        marlin.go(x, y, safeHeight, mm_per_second=zSpeed)
    
    measurements = pd.DataFrame(measurements)
    
    centerline = measurements.groupby('x')[['y']].mean().reset_index()
    centerline['c'] = 1.0
    
    model = sm.OLS(centerline.y, centerline[['c', 'x']]).fit()
    center_y_at_x0 = model.params['c'] + model.params['x'] * 0.0
    center_y_safe = model.params['c'] + model.params['x'] * -safeDistance
    
    marlin.go(cx - safeDistance, center_y_safe, safeHeight, mm_per_second=xySpeed)
    marlin.go(cx - safeDistance, center_y_safe, probeHeight, mm_per_second=zSpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    x, y, _ = marlin.probe(cx, center_y_at_x0, probeHeight, mm_per_second=probeSpeed, towards=True)
    x += 1.0
    y += model.params['x'] * 1.0
    
    marlin.go(cx - safeDistance, center_y_safe, probeHeight, mm_per_second=xySpeed)
    marlin.go(cx - safeDistance, center_y_safe, safeHeight, mm_per_second=zSpeed)
    marlin.go(x, y, safeHeight, mm_per_second=xySpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    x, y, z = marlin.probe(x, y, probeHeight, mm_per_second=probeSpeed, towards=True)
    return (x, y, z)

approxLen = 150.0
approxAngle = 180.0
N = 3

x, y, z = find_center()
circle += [{'x': x, 'y': y, 'z': z, 'angle': approxAngle}]

for i in range(N):
       
    # Now rotate the spindle
    marlin.go(x, y, safeHeight, mm_per_second=zSpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    marlin.go(x + 6, y - safeDistance, safeHeight, mm_per_second=xySpeed)
    marlin.go(x + 6, y - safeDistance, probeHeight, mm_per_second=zSpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    if 0:
        marlin.go(0, -safeDistance, probeHeight, mm_per_second=zSpeed, wait=True)
        marlin.send('M801 R0')
        
    tx, ty, _ = marlin.probe(x + 6, y, probeHeight, mm_per_second=probeSpeed, towards=True)
    assert(marlin.isZProbeTriggered())

    sx = approxLen * math.cos(math.radians(approxAngle)) + approxLen
    sy = approxLen * math.sin(math.radians(approxAngle))
    
    for j in range(1, 45 / N + 1):
        approxAngle -= 1
        ex = approxLen * math.cos(math.radians(approxAngle)) + approxLen
        ey = approxLen * math.sin(math.radians(approxAngle))    
        dx = ex - sx
        dy = ey - sy

        marlin.rotateArm(tx + dx, ty + dy, probeHeight, clockwise=True, mm_per_second=xySpeed)
        assert(marlin.isZProbeTriggered())
        
    marlin.go(tx + dx, ty + dy - safeDistance, probeHeight, mm_per_second=xySpeed)
    marlin.go(tx + dx, ty + dy - safeDistance, safeHeight, mm_per_second=zSpeed)
    marlin.go(tx + dx, ty + dy + 7.0, safeHeight, mm_per_second=xySpeed) # TODO correct for angle
    
    x, y, z = find_center()
    circle += [{'x': x, 'y': y, 'z': z, 'angle': approxAngle}]


circle = pd.DataFrame(circle)
