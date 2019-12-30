#!/usr/bin/env python2
# -*- coding: utf-8 -*-

import math, sys
import pandas as pd
import matplotlib.pyplot as plt
import statsmodels.api as sm
import seaborn as sns
from scipy.interpolate import interp1d

from marlin import Marlin

useSimulator = True

simulator = {
    'executable': '/home/peter/github/cnc-z-perpendicularity/simulator/target/debug/simulator',
    'working_directory': '/home/peter/github/cnc-z-perpendicularity/simulator',
    'fast': True,
}

marlinPort = "/dev/serial/by-id/usb-Arduino__www.arduino.cc__0042_85531303231351E0E181-if00"
marlinBaudrate = 250000

safeHeight = 10.0  # mm
rotateHeight = -5.0 # mm
safeDistance = 15.0 # mm
maxProbeDepth = 15 # mm

zSpeed = 3 # mm/s
xySpeed = 8 # mm/s
probeSpeed = 1 # mm/s

feelerGaugeWidth = 10.0 # mm
probeWidth = 4.0 # mm

###############################################################################

pd.set_option('display.max_columns', 10)
pd.set_option('display.width', 125)

###############################################################################

marlin = Marlin(simulator if useSimulator else None)
marlin.connect(marlinPort, marlinBaudrate)

if useSimulator:
    marlin.send('M800 A0.5  B0.25')
    marlin.send('M801 A1    B0.5   R0')
    marlin.send('M802 A0.5  B1     O150')
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
    print 'Error: Z probe is still triggered after trying to move up. Please check your probe.'
    sys.exit(0)

circle = []

def find_center():
    global marlin, maxProbeDepth, safeHeight
    
    cx, cy, cz = marlin.getPosition()
    assert(cz >= safeHeight)
    
    probeDepths = range(-maxProbeDepth, 0, 2)
#    probeDepths = [-1]
    
    ########################################

    front_back = []
    for side in [-1, 1]:
        y = side * safeDistance + cy
        
        for i in range(-2, 2+1):
            x = side * -i * 2 + 2 * 2 + cx
            
            _, _, z = marlin.getPosition()
            marlin.go(x, y, z, mm_per_second=xySpeed)
                
            for z in probeDepths:
                marlin.go(x, y, z, mm_per_second=zSpeed, wait=True)
                assert(not marlin.isZProbeTriggered())
            
                _, y, _ = marlin.probe(x, cy, z, mm_per_second=probeSpeed, towards=True)
                front_back += [{'x': x, 'y': y, 'z': z, 'side': side, 'ok': marlin.isZProbeTriggered()}]
                y = y + side * 1
        
                marlin.go(x, y, z, mm_per_second=xySpeed)
    
        marlin.go(x, y, safeHeight, mm_per_second=zSpeed)
    
    front_back = pd.DataFrame(front_back)

    # fit a (vertical) plane through the centerline of the gauge at different Z heights
    # Given X and Z, find the Y of the plane
    # y = c + ax + bz + epsilon
    plane = front_back.groupby(['z', 'x'])[['y']].mean().reset_index()
    plane['c'] = 1.0
    plane = sm.OLS(plane['y'], plane[['c', 'x', 'z']]).fit()
    if 0:
        print plane.summary()
    
    ########################################

    side = []
    x = cx - safeDistance
    marlin.go(x, cy, safeHeight, mm_per_second=xySpeed)
    for z in probeDepths:
        centerline = front_back[front_back['z'] == z].groupby('x')[['y']].mean().reset_index()
        centerline['c'] = 1.0        
        model = sm.OLS(centerline.y, centerline[['c', 'x']]).fit()
        center_y_at_cx = model.params['c'] + model.params['x'] * cx
        center_y_at_x = model.params['c'] + model.params['x'] * x

        marlin.go(x, center_y_at_x, z, mm_per_second=zSpeed, wait=True)
        assert(not marlin.isZProbeTriggered())
    
        x, y, _ = marlin.probe(cx, center_y_at_cx, z, mm_per_second=probeSpeed, towards=True)
        side += [{'x': x, 'y': y, 'z': z, 'side': 0, 'ok': marlin.isZProbeTriggered()}]
        x -= 1

        marlin.go(x, center_y_at_x, z, mm_per_second=xySpeed)

    marlin.go(x, center_y_at_x, safeHeight, mm_per_second=zSpeed)
    
    side = pd.DataFrame(side)
    
    ########################################
    
    tip_x_fn = interp1d(side['z'].values, side['x'].values, kind='linear', fill_value='extrapolate')
    z_start  = safeHeight
    z_target = -3.0
    x_start  = tip_x_fn(z_start) + 5.0 # TODO this is not exactly on the center line of the feeler gauge
    x_target = tip_x_fn(z_target) + 5.0
    y_start  = plane.predict(pd.DataFrame({'c': 1, 'x': x_start,  'z': z_start},  index=[0])).loc[0]
    y_target = plane.predict(pd.DataFrame({'c': 1, 'x': x_target, 'z': z_target}, index=[0])).loc[0]
        
    marlin.go(x_start, y_start, z_start, mm_per_second=xySpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    x, y, z = marlin.probe(x_target, y_target, z_target, mm_per_second=probeSpeed, towards=True)
    measurements = pd.concat([front_back, side], ignore_index=True)
    return (x, y, z, measurements)

approxLen = 150.0
approxAngle = 180.0
N = 3

x, y, z, _ = find_center()
circle += [{'x': x, 'y': y, 'z': z, 'approx_angle': approxAngle}]

for i in range(N):
       
    # Now rotate the spindle
    marlin.go(x, y, safeHeight, mm_per_second=zSpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    marlin.go(x, y - safeDistance, safeHeight, mm_per_second=xySpeed)
    marlin.go(x, y - safeDistance, rotateHeight, mm_per_second=zSpeed, wait=True)
    assert(not marlin.isZProbeTriggered())
    
    if 0:
        marlin.go(0, -safeDistance, rotateHeight, mm_per_second=zSpeed, wait=True)
        marlin.send('M801 R0')
        
    tx, ty, _ = marlin.probe(x, y, rotateHeight, mm_per_second=probeSpeed, towards=True)
    assert(marlin.isZProbeTriggered())

    sx = approxLen * math.cos(math.radians(approxAngle)) + approxLen
    sy = approxLen * math.sin(math.radians(approxAngle))
    
    for j in range(1, 45 / N + 1):
        approxAngle -= 1
        ex = approxLen * math.cos(math.radians(approxAngle)) + approxLen
        ey = approxLen * math.sin(math.radians(approxAngle))    
        dx = ex - sx
        dy = ey - sy

        marlin.rotateArm(tx + dx, ty + dy, rotateHeight, clockwise=True, mm_per_second=xySpeed)
        assert(marlin.isZProbeTriggered())
        
    marlin.go(tx + dx, ty + dy - safeDistance, rotateHeight, mm_per_second=xySpeed)
    marlin.go(tx + dx, ty + dy - safeDistance, safeHeight, mm_per_second=zSpeed)
    marlin.go(tx + dx, ty + dy + (feelerGaugeWidth + probeWidth) / 2.0, safeHeight, mm_per_second=xySpeed) # TODO correct for angle
    
    x, y, z, _ = find_center()
    circle += [{'x': x, 'y': y, 'z': z, 'approx_angle': approxAngle}]


circle = pd.DataFrame(circle)

plane = circle[['x', 'y', 'z']].copy()
plane['c'] = 1.0
plane = sm.OLS(plane['z'], plane[['c', 'x', 'y']]).fit()
slope_x = plane.params['x']
slope_y = plane.params['y']

z_plus_spindle_angle_x = math.degrees(math.atan2(plane.params['y'], 1.0))
z_plus_spindle_angle_y = -math.degrees(math.atan2(plane.params['x'], 1.0))
print 'Z plus spindle angle X:', z_plus_spindle_angle_x
print 'Z plus spindle angle Y:', z_plus_spindle_angle_y
