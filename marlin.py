#!/usr/bin/env python2
# -*- coding: utf-8 -*-

import serial
import time
import subprocess

class Marlin:
    def __init__(self, simulator=None):
        self.simulator = simulator
        
    def connect(self, port, baudrate, timeoutSeconds=30, waitSeconds=8):
        if self.simulator is not None:
            self.conn = subprocess.Popen([self.simulator['executable'], "--no-keyboard"],
                            cwd=self.simulator['working_directory'],
                            stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE)
        else:
            self.conn = serial.Serial(port, baudrate, timeout=timeoutSeconds)
            time.sleep(waitSeconds)
            self.conn.flushInput()
        
        # first command sent can be garbled, so send something that doesn't do anything
        self.send('M110 N0')
        
        # Make sure we can communicate properly now
        x, y, z = self.getPosition()
        assert(x is not None and y is not None and z is not None)

        self.send('G90')

    def close(self):
        if self.simulator is not None:
            self.conn.stdin.close()
            self.conn.stdout.close()
        else:
            self.conn.close()
        self.conn = None
        
    def go(self, x, y, z, mm_per_second=8, rapid=False, wait=False):
        command = 'G0' if rapid else 'G1'
        self.send('%s X%f Y%f Z%f F%f' % (command, x, y, z, mm_per_second * 60))
        if wait:
            self.waitUntilStopped()

    def probe(self, x, y, z, mm_per_second=8, towards=True):
        command = 'G38.2' if towards else 'G38.4'
        for attempt in range(3):
            self.send('%s X%f Y%f Z%f F%f' % (command, x, y, z, mm_per_second * 60))
            self.waitUntilStopped()
        return self.getPosition()

    def enableSteppers(self, x=True, y=True, z=True):
        self.send('M17%s%s%s' % (' X' if x else '', ' Y' if y else '', ' Z' if z else ''))

    def disableSteppers(self, x=True, y=True, z=False):
        self.send('M18%s%s%s' % (' X' if x else '', ' Y' if y else '', ' Z' if z else ''))

    def setPosition(self, x, y, z):
        self.send('G92 X%f Y%f Z%f' % (x, y, z))

    def home(self, axis='Z'):
        self.send('G28 %s' % axis)

    def waitUntilStopped(self):
        self.send('M400')
        
    def getPosition(self):
        return self._parsePosition(self.send('M114'))
        
    def isZProbeTriggered(self, pin='z_min'):
        response = self.send('M119')
        for line in response.split('\n'):
            if line.startswith(pin + ': '):
                return 'TRIGGERED' in line
        return None
        
    def send(self, line):
        outputStream = self.conn if self.simulator is None else self.conn.stdin
        outputStream.write(line.strip() + '\n')
        return self._receive()
        
    def _receive(self):
        inputStream = self.conn if self.simulator is None else self.conn.stdout
        result = ''
        while True:
            line = inputStream.readline().strip()
            if line is None:
                raise Exception('Connection closed while waiting for response')
            if line == 'ok':
                break
            if not line.startswith("echo:busy:") and not line.startswith("busy:") and not line.startswith("//"):
                result += line + '\n'
        return result

    def _parsePosition(self, line):
        fields = {}
        while ':' in line:
            label, line = line.split(':', 1)
            value, line = line.split(' ', 1) if ' ' in line else (line, '')
            line = line.strip()
            if label not in fields:
                fields[label] = float(value)
        return (fields.get('X', None), fields.get('Y', None), fields.get('Z', None))
