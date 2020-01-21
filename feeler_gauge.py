#!/usr/bin/env python2
# -*- coding: utf-8 -*-

import math, sys
import pandas as pd
import matplotlib.pyplot as plt
import statsmodels.api as sm
from scipy.interpolate import interp1d
from skimage.measure import EllipseModel
import matplotlib

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

feelerGaugeWidth = 13.0 # mm
feelerGaugeLength = 89.0 # mm
probeWidth = 4.0 # mm

approxLen = 150.0
approxAngle = 180.0

numAngles = 6 # should be a multiple of 3
#probeDepths = range(-maxProbeDepth, 0, 2)
probeDepths = range(-maxProbeDepth, 0, 4)
xOffsets = range(-2, 2+1)

###############################################################################

assert(numAngles % 3 == 0)
assert(numAngles >= 3)
assert(len(probeDepths) >= 2)
assert(len(xOffsets) >= 2)

pd.set_option('display.max_columns', 10)
pd.set_option('display.width', 125)

###############################################################################

marlin = Marlin(simulator if useSimulator else None)
marlin.connect(marlinPort, marlinBaudrate)

if useSimulator:
    startX = 500 + math.cos(math.radians(approxAngle)) * approxLen - (feelerGaugeLength / 2.0 - 3.0)
    startY = 250 + math.sin(math.radians(approxAngle)) * approxLen
    if 0:
        marlin.send('M800 A0.5  B0.25')
        marlin.send('M801 A1    B0.5   R%d' % (approxAngle - 180))
        marlin.send('M802 A0.5  B1     O%f' % approxLen)
        marlin.send('G1 X%d Y%d' % (startX, startY))
    elif 0:
        marlin.send('M800 A0.05 B0.19')
        marlin.send('M801 A0.13 B0.23 R%d' % (approxAngle - 180))
        marlin.send('M802 A0.03 B0.07 O%f' % approxLen)
        marlin.send('G1 X%d Y%d' % (startX, startY))
    elif 0:
        marlin.send('M800 A0 B0')
        marlin.send('M801 A0 B0 R%d' % (approxAngle - 180))
        marlin.send('M802 A0.03 B0.07 O%f' % approxLen)
        marlin.send('G1 X%d Y%d' % (startX, startY))
    else:
        marlin.send('M800 A0 B0')
        marlin.send('M801 A0 B0 R%d' % (approxAngle - 180))
        marlin.send('M802 A0 B0 O%f' % approxLen)
        marlin.send('G1 X%d Y%d' % (startX, startY))
    marlin.home()

if not marlin.isZProbeTriggered():
    print 'Error: Z probe is not triggered.'
    print 'Make sure the router is in the correct starting position, and that the Z probe is triggered.'
    sys.exit(0)

marlin.enableSteppers()
marlin.setPosition(0, 0, 0)
marlin.go(0, 0, safeHeight, mm_per_second=zSpeed, wait=True)

if marlin.isZProbeTriggered():
    print 'Error: Z probe is still triggered after trying to move up. Please check your probe.'
    sys.exit(0)

circle = []
measurements = []

def find_center():
    global marlin, maxProbeDepth, safeHeight, probeDepths, xOffsets, approxAngle
    
    cx, cy, cz = marlin.getPosition()
    assert(cz >= safeHeight)
    
    ########################################

    front_back = []
    for side in [-1, 1]:
        y = side * safeDistance + cy
        
        for i in xOffsets:
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
    measurements['approx_angle'] = approxAngle
    return (x, y, z, measurements)

x, y, z, m = find_center()
circle += [{'x': x, 'y': y, 'z': z, 'approx_angle': approxAngle}]
measurements += [m]

for side in [-1, 0, 1]:
    offsetX = -safeDistance if side == 0 else 5
    offsetY = side * safeDistance
    
    centerX = 5 if side == 0 else 0
    centerY = -side * (feelerGaugeWidth + probeWidth) / 2.0
    
    rotateX = x + offsetX
    rotateY = y + offsetY
    
    for i in range(numAngles / 3):           
        # Now rotate the spindle
        marlin.go(x, y, safeHeight, mm_per_second=zSpeed, wait=True)
        assert(not marlin.isZProbeTriggered())
        
        marlin.go(rotateX, rotateY, safeHeight, mm_per_second=xySpeed)
        marlin.go(rotateX, rotateY, rotateHeight, mm_per_second=zSpeed, wait=True)
        assert(not marlin.isZProbeTriggered())
        
        tx, ty, _ = marlin.probe(x, y, rotateHeight, mm_per_second=probeSpeed, towards=True)
        assert(marlin.isZProbeTriggered())
    
        sx = approxLen * math.cos(math.radians(approxAngle)) + approxLen
        sy = approxLen * math.sin(math.radians(approxAngle))
        
        for j in range(90 / (numAngles / 3)):
            approxAngle = (approxAngle - 1 + 360) % 360
            ex = approxLen * math.cos(math.radians(approxAngle)) + approxLen
            ey = approxLen * math.sin(math.radians(approxAngle))    
            dx = ex - sx
            dy = ey - sy
    
            marlin.rotateArm(tx + dx, ty + dy, rotateHeight, clockwise=True, mm_per_second=xySpeed)
            assert(marlin.isZProbeTriggered())
            
        marlin.go(tx + dx + offsetX, ty + dy + offsetY, rotateHeight, mm_per_second=xySpeed)
        marlin.go(tx + dx + offsetX, ty + dy + offsetY, safeHeight, mm_per_second=zSpeed)
        marlin.go(tx + dx + centerX, ty + dy + centerY, safeHeight, mm_per_second=xySpeed) # TODO correct for angle
        
        x, y, z, m = find_center()
        circle += [{'x': x, 'y': y, 'z': z, 'approx_angle': approxAngle}]
        measurements += [m]


circle = pd.DataFrame(circle).set_index('approx_angle').sort_index()
measurements = pd.concat(measurements, ignore_index=True)

plane = circle[['x', 'y', 'z']].copy()
plane['c'] = 1.0
plane = sm.OLS(plane['z'], plane[['c', 'x', 'y']]).fit()
slope_x = plane.params['x']
slope_y = plane.params['y']

spindle_angle_x = math.degrees(math.atan2(plane.params['y'], 1.0))
spindle_angle_y = -math.degrees(math.atan2(plane.params['x'], 1.0))
print 'Spindle angle around X: %8.4f degrees off perpendicular' % spindle_angle_x
print 'Spindle angle around Y: %8.4f degrees off perpendicular' % spindle_angle_y

if 0:
    from matplotlib.patches import Ellipse
    
    ell = EllipseModel()
    ell.estimate(circle[['x', 'y']].values)
    xc, yc, a, b, theta = ell.params
    
    fig = plt.figure()
    ax = plt.gca()
    ax.set_aspect('equal')
    ax.plot(circle.x.values, circle.y.values, '.')
    ax.scatter(xc, yc, color='red', s=100)
    ax.add_patch(matplotlib.patches.Ellipse((xc, yc), 2*a, 2*b, theta*180/math.pi, edgecolor='red', facecolor='none'))
    plt.show()

probe_center_line = {}
for angle in measurements.approx_angle.unique():
    x = measurements[(measurements.side != 0) & (measurements.approx_angle == angle)].x.min()
    front = measurements[(measurements.side == -1) & (measurements.approx_angle == angle) & (measurements.x == x)].set_index('z')['y']
    back = measurements[(measurements.side == 1) & (measurements.approx_angle == angle) & (measurements.x == x)].set_index('z')['y']
    probe_center_line[angle] = (front + back) / 2.0
probe_center_line = pd.DataFrame(probe_center_line)

spindle_center_line = {}
for angle in [0, 45, 135]:
    spindle_center_line[angle] = probe_center_line[[angle, angle + 180]].mean(axis=1)
spindle_center_line = pd.DataFrame(spindle_center_line).reset_index()
spindle_center_line['c'] = 1.0

# For spindle center line at angle 0: 
# arm is parallel with the X axis, can determine rotation around X axis
#
# delta_y = delta_z * (sin(z) + cos(z) * tan(s))
# where z = angle of z axis off perpendicular, CCW
#       s = angle of spindle off perpendicular, CW
#
# We also have this relation:
#   s = spindle_angle_x + z
#
# So delta_y = delta_z * (sin(z) + cos(z) * tan(z_plus_spindle_angle_x + z))

# If z = 0:
#   delta_y = delta_z * (0 + 1 * tan(s))
#   delta_y / delta_z = tan(s)
#   tan(s) = delta_y / delta_z
#   s = atan2(delta_y, delta_z)

model = sm.OLS(spindle_center_line[0], spindle_center_line[['c', 'z']]).fit()
if 0:
    print model.summary()

delta_z = 1.0
delta_y = model.params['z'] * delta_z

z_axis_to_spindle_angle = -math.degrees(math.atan2(delta_y, delta_z))

print 'Z axis angle around X:  %8.4f degrees off perpendicular' % (spindle_angle_x - z_axis_to_spindle_angle)
print 'Z axis to spindle in X: %8.4f' % z_axis_to_spindle_angle
